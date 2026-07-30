use std::fs;
use std::path::PathBuf;
use rusqlite::Connection;
use sha2::{Digest, Sha256};
use tauri::{AppHandle, Manager};

pub struct DbPathState {
    pub path: PathBuf,
}

pub fn hash_password(password: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(password.as_bytes());
    let result = hasher.finalize();
    format!("{:x}", result)
}

pub fn init_db(app_handle: &AppHandle) -> Result<PathBuf, String> {
    let app_dir = app_handle.path().app_data_dir().map_err(|e| e.to_string())?;
    fs::create_dir_all(&app_dir).map_err(|e| e.to_string())?;
    let db_path = app_dir.join("mealdesk.db");
    
    let conn = Connection::open(&db_path).map_err(|e| e.to_string())?;
    
    create_tables(&conn)?;
    migrate_db(&conn)?;
    seed_default_data(&conn)?;
    
    Ok(db_path)
}
#[cfg(test)]
pub fn setup_test_db() -> Connection {
    let conn = Connection::open_in_memory().unwrap();
    create_tables(&conn).unwrap();
    migrate_db(&conn).unwrap();
    seed_default_data(&conn).unwrap();
    conn
}
fn migrate_db(conn: &Connection) -> Result<(), String> {
    // 1. Check if database is in a broken state from a previous partial/incorrect migration (referencing orders_old)
    let is_broken: i64 = conn.query_row(
        "SELECT COUNT(*) FROM sqlite_master WHERE sql LIKE '%orders_old%'",
        [],
        |row| row.get(0),
    ).unwrap_or(0);

    if is_broken > 0 {
        // Drop all tables to trigger a clean recreate
        let tables_to_drop = [
            "kot_items", "kot", "order_items", "orders", "inventory", 
            "purchase_history", "products", "categories", "customers", 
            "tables", "users", "restaurant_info"
        ];
        conn.execute("PRAGMA foreign_keys = OFF;", []).map_err(|e| e.to_string())?;
        for table in &tables_to_drop {
            let _ = conn.execute(&format!("DROP TABLE IF EXISTS {};", table), []);
        }
        conn.execute("PRAGMA foreign_keys = ON;", []).map_err(|e| e.to_string())?;
        
        // Recreate tables and seed data immediately
        create_tables(conn)?;
        seed_default_data(conn)?;
        return Ok(());
    }

    // 2. Correct schema migration logic
    let sql: String = conn.query_row(
        "SELECT sql FROM sqlite_master WHERE type = 'table' AND name = 'orders'",
        [],
        |row| row.get(0),
    ).unwrap_or_default();

    if !sql.is_empty() && (!sql.contains("'Draft'") || !sql.contains("'NC'")) {
        // Run migration using the safe SQLite table recreation template
        conn.execute("PRAGMA foreign_keys = OFF;", []).map_err(|e| e.to_string())?;

        // Create the new orders_new table with updated constraint
        conn.execute(
            "CREATE TABLE orders_new (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                table_id INTEGER REFERENCES tables(id) ON DELETE SET NULL,
                customer_id INTEGER REFERENCES customers(id) ON DELETE SET NULL,
                subtotal REAL NOT NULL,
                tax REAL NOT NULL,
                discount REAL NOT NULL DEFAULT 0.0,
                service_charge REAL NOT NULL DEFAULT 0.0,
                round_off REAL NOT NULL DEFAULT 0.0,
                total REAL NOT NULL,
                status TEXT NOT NULL CHECK (status IN ('Draft', 'Pending', 'Billed', 'Completed', 'Cancelled')),
                payment_mode TEXT CHECK (payment_mode IN ('Cash', 'UPI', 'Card', 'Mixed', 'NC', 'None')),
                notes TEXT,
                hold_name TEXT,
                created_at TEXT NOT NULL
            );",
            [],
        ).map_err(|e| e.to_string())?;

        // Copy records from orders to orders_new
        conn.execute(
            "INSERT INTO orders_new (id, table_id, customer_id, subtotal, tax, discount, service_charge, round_off, total, status, payment_mode, notes, hold_name, created_at)
             SELECT id, table_id, customer_id, subtotal, tax, discount, service_charge, round_off, total, status, payment_mode, notes, hold_name, created_at FROM orders;",
            [],
        ).map_err(|e| e.to_string())?;

        // Drop the old orders table
        conn.execute("DROP TABLE orders;", []).map_err(|e| e.to_string())?;

        // Rename orders_new to orders
        conn.execute("ALTER TABLE orders_new RENAME TO orders;", []).map_err(|e| e.to_string())?;

        conn.execute("PRAGMA foreign_keys = ON;", []).map_err(|e| e.to_string())?;
    }

    // Migration for kot table to add print_count column
    let kot_sql: String = conn.query_row(
        "SELECT sql FROM sqlite_master WHERE type = 'table' AND name = 'kot'",
        [],
        |row| row.get(0),
    ).unwrap_or_default();

    if !kot_sql.is_empty() && !kot_sql.contains("print_count") {
        conn.execute(
            "ALTER TABLE kot ADD COLUMN print_count INTEGER NOT NULL DEFAULT 0;",
            [],
        ).map_err(|e| e.to_string())?;
    }

    // Migration for orders table to add cancellation columns if not present
    let orders_sql: String = conn.query_row(
        "SELECT sql FROM sqlite_master WHERE type = 'table' AND name = 'orders'",
        [],
        |row| row.get(0),
    ).unwrap_or_default();

    if !orders_sql.is_empty() && !orders_sql.contains("cancelled_by") {
        conn.execute(
            "ALTER TABLE orders ADD COLUMN cancelled_by TEXT;",
            [],
        ).map_err(|e| e.to_string())?;
        conn.execute(
            "ALTER TABLE orders ADD COLUMN cancelled_at TEXT;",
            [],
        ).map_err(|e| e.to_string())?;
        conn.execute(
            "ALTER TABLE orders ADD COLUMN cancel_reason TEXT;",
            [],
        ).map_err(|e| e.to_string())?;
    }

    // Migration for order_items table to add kot_id column if not present
    let order_items_sql: String = conn.query_row(
        "SELECT sql FROM sqlite_master WHERE type = 'table' AND name = 'order_items'",
        [],
        |row| row.get(0),
    ).unwrap_or_default();

    if !order_items_sql.is_empty() && !order_items_sql.contains("kot_id") {
        conn.execute(
            "ALTER TABLE order_items ADD COLUMN kot_id INTEGER REFERENCES kot(id) ON DELETE SET NULL;",
            [],
        ).map_err(|e| e.to_string())?;
    }

    // Create order_item_cancellations table if not exists
    conn.execute(
        "CREATE TABLE IF NOT EXISTS order_item_cancellations (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            order_item_id INTEGER NOT NULL REFERENCES order_items(id) ON DELETE CASCADE,
            quantity INTEGER NOT NULL,
            reason TEXT NOT NULL,
            cancelled_by TEXT NOT NULL,
            created_at TEXT NOT NULL
        );",
        [],
    ).map_err(|e| e.to_string())?;

    // Create audit_logs table if not exists
    conn.execute(
        "CREATE TABLE IF NOT EXISTS audit_logs (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            username TEXT NOT NULL,
            action TEXT NOT NULL,
            target_type TEXT NOT NULL,
            target_id INTEGER,
            details TEXT,
            created_at TEXT NOT NULL
        );",
        [],
    ).map_err(|e| e.to_string())?;

    Ok(())
}

fn create_tables(conn: &Connection) -> Result<(), String> {
    // Enable foreign keys
    conn.execute("PRAGMA foreign_keys = ON;", []).map_err(|e| e.to_string())?;

    conn.execute(
        "CREATE TABLE IF NOT EXISTS users (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            username TEXT UNIQUE NOT NULL,
            password_hash TEXT NOT NULL,
            role TEXT NOT NULL CHECK (role IN ('Owner', 'Manager', 'Cashier'))
        );",
        [],
    ).map_err(|e| e.to_string())?;

    conn.execute(
        "CREATE TABLE IF NOT EXISTS restaurant_info (
            id INTEGER PRIMARY KEY CHECK (id = 1),
            name TEXT NOT NULL,
            logo TEXT,
            gstin TEXT,
            address TEXT,
            phone TEXT,
            email TEXT,
            receipt_footer TEXT
        );",
        [],
    ).map_err(|e| e.to_string())?;

    conn.execute(
        "CREATE TABLE IF NOT EXISTS categories (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            name TEXT UNIQUE NOT NULL,
            description TEXT
        );",
        [],
    ).map_err(|e| e.to_string())?;

    conn.execute(
        "CREATE TABLE IF NOT EXISTS products (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            category_id INTEGER NOT NULL,
            name TEXT NOT NULL,
            price REAL NOT NULL,
            gst_rate REAL NOT NULL DEFAULT 0.0,
            image_path TEXT,
            is_available INTEGER NOT NULL DEFAULT 1,
            FOREIGN KEY (category_id) REFERENCES categories (id) ON DELETE RESTRICT
        );",
        [],
    ).map_err(|e| e.to_string())?;

    conn.execute(
        "CREATE TABLE IF NOT EXISTS customers (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            name TEXT NOT NULL,
            phone TEXT UNIQUE NOT NULL,
            email TEXT,
            loyalty_points INTEGER DEFAULT 0
        );",
        [],
    ).map_err(|e| e.to_string())?;

    conn.execute(
        "CREATE TABLE IF NOT EXISTS tables (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            name TEXT UNIQUE NOT NULL,
            status TEXT NOT NULL DEFAULT 'Free' CHECK (status IN ('Free', 'Occupied', 'Billed')),
            merged_into INTEGER REFERENCES tables(id) ON DELETE SET NULL,
            current_order_id INTEGER
        );",
        [],
    ).map_err(|e| e.to_string())?;

    conn.execute(
        "CREATE TABLE IF NOT EXISTS orders (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            table_id INTEGER REFERENCES tables(id) ON DELETE SET NULL,
            customer_id INTEGER REFERENCES customers(id) ON DELETE SET NULL,
            subtotal REAL NOT NULL,
            tax REAL NOT NULL,
            discount REAL NOT NULL DEFAULT 0.0,
            service_charge REAL NOT NULL DEFAULT 0.0,
            round_off REAL NOT NULL DEFAULT 0.0,
            total REAL NOT NULL,
            status TEXT NOT NULL CHECK (status IN ('Draft', 'Pending', 'Billed', 'Completed', 'Cancelled')),
            payment_mode TEXT CHECK (payment_mode IN ('Cash', 'UPI', 'Card', 'Mixed', 'NC', 'None')),
            notes TEXT,
            hold_name TEXT,
            created_at TEXT NOT NULL
        );",
        [],
    ).map_err(|e| e.to_string())?;

    conn.execute(
        "CREATE TABLE IF NOT EXISTS order_items (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            order_id INTEGER NOT NULL REFERENCES orders(id) ON DELETE CASCADE,
            product_id INTEGER NOT NULL REFERENCES products(id) ON DELETE CASCADE,
            name TEXT NOT NULL,
            quantity INTEGER NOT NULL,
            price REAL NOT NULL,
            gst_rate REAL NOT NULL,
            notes TEXT
        );",
        [],
    ).map_err(|e| e.to_string())?;

    conn.execute(
        "CREATE TABLE IF NOT EXISTS kot (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            order_id INTEGER NOT NULL REFERENCES orders(id) ON DELETE CASCADE,
            status TEXT NOT NULL CHECK (status IN ('Pending', 'Preparing', 'Ready', 'Completed')),
            print_count INTEGER NOT NULL DEFAULT 0,
            created_at TEXT NOT NULL
        );",
        [],
    ).map_err(|e| e.to_string())?;

    conn.execute(
        "CREATE TABLE IF NOT EXISTS kot_items (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            kot_id INTEGER NOT NULL REFERENCES kot(id) ON DELETE CASCADE,
            product_id INTEGER NOT NULL REFERENCES products(id) ON DELETE CASCADE,
            quantity INTEGER NOT NULL,
            notes TEXT
        );",
        [],
    ).map_err(|e| e.to_string())?;

    conn.execute(
        "CREATE TABLE IF NOT EXISTS inventory (
            product_id INTEGER PRIMARY KEY REFERENCES products(id) ON DELETE CASCADE,
            stock_qty INTEGER NOT NULL DEFAULT 0,
            low_stock_threshold INTEGER NOT NULL DEFAULT 5
        );",
        [],
    ).map_err(|e| e.to_string())?;

    conn.execute(
        "CREATE TABLE IF NOT EXISTS purchase_history (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            product_id INTEGER NOT NULL REFERENCES products(id) ON DELETE CASCADE,
            quantity INTEGER NOT NULL,
            supplier TEXT,
            unit_price REAL NOT NULL,
            date TEXT NOT NULL
        );",
        [],
    ).map_err(|e| e.to_string())?;

    Ok(())
}

fn seed_default_data(conn: &Connection) -> Result<(), String> {
    // Seed default users if empty
    let user_count: i64 = conn
        .query_row("SELECT COUNT(*) FROM users", [], |row| row.get(0))
        .map_err(|e| e.to_string())?;

    if user_count == 0 {
        let default_users = vec![
            ("owner", hash_password("owner123"), "Owner"),
            ("manager", hash_password("manager123"), "Manager"),
            ("cashier", hash_password("cashier123"), "Cashier"),
        ];

        for (username, password_hash, role) in default_users {
            conn.execute(
                "INSERT INTO users (username, password_hash, role) VALUES (?1, ?2, ?3)",
                rusqlite::params![username, password_hash, role],
            )
            .map_err(|e| e.to_string())?;
        }
    }

    // Seed default restaurant info if empty
    let info_count: i64 = conn
        .query_row("SELECT COUNT(*) FROM restaurant_info", [], |row| row.get(0))
        .map_err(|e| e.to_string())?;

    if info_count == 0 {
        conn.execute(
            "INSERT INTO restaurant_info (id, name, logo, gstin, address, phone, email, receipt_footer) 
             VALUES (1, 'MealDesk Bistro', '', '27AAAAA1111A1Z1', '123 Foodie Street, Gourmet City', '+1234567890', 'info@mealdesk.com', 'Thank you for dining with us!')",
            [],
        )
        .map_err(|e| e.to_string())?;
    }

    // Seed sample categories if empty
    let cat_count: i64 = conn
        .query_row("SELECT COUNT(*) FROM categories", [], |row| row.get(0))
        .map_err(|e| e.to_string())?;

    if cat_count == 0 {
        let categories = vec![
            ("Beverages", "Cold & hot drinks"),
            ("Starters", "Appetizers and quick bites"),
            ("Main Course", "Delicious main dishes"),
            ("Desserts", "Sweet treats"),
        ];

        for (name, desc) in categories {
            conn.execute(
                "INSERT INTO categories (name, description) VALUES (?1, ?2)",
                [name, desc],
            )
            .map_err(|e| e.to_string())?;
        }

        // Add some default products linked to these categories
        let sample_products = vec![
            ("Iced Latte", 1, 150.00, 18.0),
            ("Masala Chai", 1, 40.00, 5.0),
            ("Paneer Tikka", 2, 280.00, 18.0),
            ("Chicken Wings", 2, 320.00, 18.0),
            ("Butter Chicken with Naan", 3, 420.00, 18.0),
            ("Veg Fried Rice", 3, 240.00, 18.0),
            ("Chocolate Brownie", 4, 180.00, 18.0),
            ("Gulab Jamun (2 pcs)", 4, 80.00, 5.0),
        ];

        for (name, cat_id, price, gst) in sample_products {
            conn.execute(
                "INSERT INTO products (category_id, name, price, gst_rate, is_available) VALUES (?1, ?2, ?3, ?4, 1)",
                rusqlite::params![cat_id, name, price, gst],
            )
            .map_err(|e| e.to_string())?;
        }
    }

    // Seed default tables if empty
    let table_count: i64 = conn
        .query_row("SELECT COUNT(*) FROM tables", [], |row| row.get(0))
        .map_err(|e| e.to_string())?;

    if table_count == 0 {
        for i in 1..=12 {
            conn.execute(
                "INSERT INTO tables (name, status) VALUES (?1, 'Free')",
                [format!("Table {}", i)],
            )
            .map_err(|e| e.to_string())?;
        }
    }

    // Seed default inventory for existing products
    let inv_count: i64 = conn
        .query_row("SELECT COUNT(*) FROM inventory", [], |row| row.get(0))
        .map_err(|e| e.to_string())?;

    if inv_count == 0 {
        let mut stmt = conn
            .prepare("SELECT id FROM products")
            .map_err(|e| e.to_string())?;
        let prod_ids = stmt
            .query_map([], |row| row.get::<_, i64>(0))
            .map_err(|e| e.to_string())?;

        for pid_res in prod_ids {
            let pid = pid_res.map_err(|e| e.to_string())?;
            conn.execute(
                "INSERT INTO inventory (product_id, stock_qty, low_stock_threshold) VALUES (?1, 50, 5)",
                rusqlite::params![pid],
            )
            .map_err(|e| e.to_string())?;
        }
    }

    Ok(())
}
