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
    eprintln!("Database initialized at: {:?}", db_path);
    
    let conn = Connection::open(&db_path).map_err(|e| e.to_string())?;
    
    // Run migration first (handles existing table renaming and schema updates)
    migrate_db(&conn)?;
    seed_default_data(&conn)?;
    
    Ok(db_path)
}

#[cfg(test)]
pub fn setup_test_db() -> Connection {
    let conn = Connection::open_in_memory().unwrap();
    migrate_db(&conn).unwrap();
    seed_default_data(&conn).unwrap();
    conn
}

fn migrate_db(conn: &Connection) -> Result<(), String> {
    let orders_table_exists: i64 = conn.query_row(
        "SELECT COUNT(*) FROM sqlite_master WHERE type = 'table' AND name = 'orders'",
        [],
        |row| row.get(0),
    ).unwrap_or(0);

    let bills_table_exists: i64 = conn.query_row(
        "SELECT COUNT(*) FROM sqlite_master WHERE type = 'table' AND name = 'bills'",
        [],
        |row| row.get(0),
    ).unwrap_or(0);

    if orders_table_exists > 0 && bills_table_exists == 0 {
        // Run schema migration
        conn.execute("PRAGMA foreign_keys = OFF;", []).map_err(|e| e.to_string())?;

        // 1. Rename orders and tables to old
        conn.execute("ALTER TABLE orders RENAME TO orders_old;", []).map_err(|e| e.to_string())?;
        conn.execute("ALTER TABLE tables RENAME TO tables_old;", []).map_err(|e| e.to_string())?;

        // 2. Create the new tables
        create_tables(conn)?;

        // 3. Migrate tables data (drop current_order_id)
        conn.execute(
            "INSERT INTO tables (id, name, status, merged_into) 
             SELECT id, name, status, merged_into FROM tables_old;",
            [],
        ).map_err(|e| e.to_string())?;

        // 4. Migrate orders data (drop subtotal, tax, discount, service_charge, round_off, total, payment_mode, hold_name)
        // Map 'Draft' status to 'Pending'
        conn.execute(
            "INSERT INTO orders (id, table_id, customer_id, status, notes, created_at, cancelled_by, cancelled_at, cancel_reason)
             SELECT id, table_id, customer_id,
                    CASE WHEN status = 'Draft' THEN 'Pending' ELSE status END,
                    notes, created_at, cancelled_by, cancelled_at, cancel_reason
             FROM orders_old;",
            [],
        ).map_err(|e| e.to_string())?;

        // 5. Migrate bills data using historical totals from orders_old
        conn.execute(
            "INSERT INTO bills (order_id, bill_number, subtotal, discount, tax, service_charge, round_off, total, status, created_at, billed_at)
             SELECT id, 'BILL-' || id, subtotal, discount, tax, service_charge, round_off, total,
                    CASE WHEN status = 'Completed' THEN 'Paid'
                         WHEN status = 'Billed' THEN 'Unpaid'
                         ELSE 'Cancelled' END,
                    created_at, created_at
             FROM orders_old
             WHERE status IN ('Billed', 'Completed', 'Cancelled');",
            [],
        ).map_err(|e| e.to_string())?;

        // 6. Migrate payments data from orders_old
        conn.execute(
            "INSERT INTO payments (bill_id, payment_method, amount, created_at)
             SELECT b.id,
                    CASE WHEN o.payment_mode IN ('Cash', 'UPI', 'Card', 'NC') THEN o.payment_mode ELSE 'Cash' END,
                    b.total,
                    o.created_at
             FROM orders_old o
             JOIN bills b ON b.order_id = o.id
             WHERE o.status = 'Completed' AND o.payment_mode IS NOT NULL AND o.payment_mode != 'None';",
            [],
        ).map_err(|e| e.to_string())?;

        // 7. Drop old tables
        conn.execute("DROP TABLE orders_old;", []).map_err(|e| e.to_string())?;
        conn.execute("DROP TABLE tables_old;", []).map_err(|e| e.to_string())?;

        // 8. Drop inventory and purchase history tables if they exist
        let _ = conn.execute("DROP TABLE IF EXISTS inventory;", []);
        let _ = conn.execute("DROP TABLE IF EXISTS purchase_history;", []);

        conn.execute("PRAGMA foreign_keys = ON;", []).map_err(|e| e.to_string())?;
    } else {
        // Fresh database or already migrated
        create_tables(conn)?;
        // Make sure legacy inventory tables are deleted
        let _ = conn.execute("DROP TABLE IF EXISTS inventory;", []);
        let _ = conn.execute("DROP TABLE IF EXISTS purchase_history;", []);
    }

    Ok(())
}

fn create_tables(conn: &Connection) -> Result<(), String> {
    // Enable foreign keys
    conn.execute("PRAGMA foreign_keys = ON;", []).map_err(|e| e.to_string())?;

    conn.execute(
        "CREATE TABLE IF NOT EXISTS users (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            username TEXT UNIQUE NOT NULL,
            password_hash TEXT NOT NULL
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
            merged_into INTEGER REFERENCES tables(id) ON DELETE SET NULL
        );",
        [],
    ).map_err(|e| e.to_string())?;

    conn.execute(
        "CREATE TABLE IF NOT EXISTS orders (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            table_id INTEGER REFERENCES tables(id) ON DELETE SET NULL,
            customer_id INTEGER REFERENCES customers(id) ON DELETE SET NULL,
            status TEXT NOT NULL CHECK (status IN ('Pending', 'Billed', 'Completed', 'Cancelled')),
            notes TEXT,
            created_at TEXT NOT NULL,
            cancelled_by TEXT,
            cancelled_at TEXT,
            cancel_reason TEXT
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
        "CREATE TABLE IF NOT EXISTS order_items (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            order_id INTEGER NOT NULL REFERENCES orders(id) ON DELETE CASCADE,
            product_id INTEGER NOT NULL REFERENCES products(id) ON DELETE CASCADE,
            name TEXT NOT NULL,
            quantity INTEGER NOT NULL,
            price REAL NOT NULL,
            gst_rate REAL NOT NULL,
            notes TEXT,
            kot_id INTEGER REFERENCES kot(id) ON DELETE SET NULL
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

    conn.execute(
        "CREATE TABLE IF NOT EXISTS bills (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            order_id INTEGER UNIQUE NOT NULL REFERENCES orders(id) ON DELETE CASCADE,
            bill_number TEXT UNIQUE NOT NULL,
            subtotal REAL NOT NULL,
            discount REAL NOT NULL DEFAULT 0.0,
            tax REAL NOT NULL,
            service_charge REAL NOT NULL DEFAULT 0.0,
            round_off REAL NOT NULL DEFAULT 0.0,
            total REAL NOT NULL,
            status TEXT NOT NULL CHECK (status IN ('Unpaid', 'Paid', 'Cancelled')),
            created_at TEXT NOT NULL,
            billed_at TEXT NOT NULL
        );",
        [],
    ).map_err(|e| e.to_string())?;

    conn.execute(
        "CREATE TABLE IF NOT EXISTS payments (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            bill_id INTEGER NOT NULL REFERENCES bills(id) ON DELETE CASCADE,
            payment_method TEXT NOT NULL CHECK (payment_method IN ('Cash', 'UPI', 'Card', 'NC')),
            amount REAL NOT NULL,
            created_at TEXT NOT NULL
        );",
        [],
    ).map_err(|e| e.to_string())?;

    Ok(())
}

fn seed_default_data(conn: &Connection) -> Result<(), String> {
    // Seed default admin if empty/does not exist
    let admin_count: i64 = conn
        .query_row("SELECT COUNT(*) FROM users WHERE username = 'admin'", [], |row| row.get(0))
        .unwrap_or(0);

    if admin_count == 0 {
        conn.execute(
            "INSERT INTO users (username, password_hash) VALUES (?1, ?2)",
            rusqlite::params!["admin", hash_password("admin123")],
        )
        .map_err(|e| e.to_string())?;
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

    Ok(())
}
