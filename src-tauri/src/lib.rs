mod db;

use serde::{Deserialize, Serialize};
use tauri::Manager;
use rusqlite::params;

// Data Structures
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct UserInfo {
    pub username: String,
    pub role: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct RestaurantInfo {
    pub name: String,
    pub logo: Option<String>,
    pub gstin: Option<String>,
    pub address: Option<String>,
    pub phone: Option<String>,
    pub email: Option<String>,
    pub receipt_footer: Option<String>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Category {
    pub id: i64,
    pub name: String,
    pub description: Option<String>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Product {
    pub id: i64,
    pub category_id: i64,
    pub name: String,
    pub price: f64,
    pub gst_rate: f64,
    pub image_path: Option<String>,
    pub is_available: bool,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct OrderItemInput {
    pub product_id: i64,
    pub name: String,
    pub quantity: i64,
    pub price: f64,
    pub gst_rate: f64,
    pub notes: Option<String>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct OrderHeader {
    pub id: i64,
    pub table_id: Option<i64>,
    pub table_name: Option<String>,
    pub customer_id: Option<i64>,
    pub customer_name: Option<String>,
    pub subtotal: f64,
    pub tax: f64,
    pub discount: f64,
    pub service_charge: f64,
    pub round_off: f64,
    pub total: f64,
    pub status: String,
    pub payment_mode: Option<String>,
    pub notes: Option<String>,
    pub hold_name: Option<String>,
    pub created_at: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct OrderItemOutput {
    pub id: i64,
    pub product_id: i64,
    pub name: String,
    pub quantity: i64,
    pub price: f64,
    pub gst_rate: f64,
    pub notes: Option<String>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct OrderOutput {
    pub header: OrderHeader,
    pub items: Vec<OrderItemOutput>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct TableDetails {
    pub id: i64,
    pub name: String,
    pub status: String,
    pub merged_into: Option<i64>,
    pub current_order_id: Option<i64>,
    pub current_order_total: Option<f64>,
    pub current_order_hold_name: Option<String>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct KotItemOutput {
    pub id: i64,
    pub product_id: i64,
    pub product_name: String,
    pub quantity: i64,
    pub notes: Option<String>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct KotOutput {
    pub id: i64,
    pub order_id: i64,
    pub table_name: Option<String>,
    pub status: String,
    pub created_at: String,
    pub items: Vec<KotItemOutput>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct InventoryOutput {
    pub product_id: i64,
    pub product_name: String,
    pub category_name: String,
    pub price: f64,
    pub stock_qty: i64,
    pub low_stock_threshold: i64,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct CustomerDetails {
    pub id: i64,
    pub name: String,
    pub phone: String,
    pub email: Option<String>,
    pub loyalty_points: i64,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct SalesReportOutput {
    pub total_sales: f64,
    pub total_tax: f64,
    pub order_count: i64,
    pub average_ticket: f64,
}

// Commands
#[tauri::command]
fn greet(name: &str) -> String {
    format!("Hello, {}! You've been greeted from Rust!", name)
}

#[tauri::command]
fn login(
    username: String,
    password: Option<String>,
    state: tauri::State<'_, db::DbPathState>,
) -> Result<UserInfo, String> {
    let conn = rusqlite::Connection::open(&state.path).map_err(|e| e.to_string())?;
    let password = password.unwrap_or_default();
    let hashed = db::hash_password(&password);
    
    let mut stmt = conn
        .prepare("SELECT username, role FROM users WHERE username = ?1 AND password_hash = ?2")
        .map_err(|e| e.to_string())?;
    
    let user_info = stmt
        .query_row([username, hashed], |row| {
            Ok(UserInfo {
                username: row.get(0)?,
                role: row.get(1)?,
            })
        })
        .map_err(|e| format!("Invalid username or password: {}", e))?;
        
    Ok(user_info)
}

#[tauri::command]
fn get_restaurant_info(state: tauri::State<'_, db::DbPathState>) -> Result<RestaurantInfo, String> {
    let conn = rusqlite::Connection::open(&state.path).map_err(|e| e.to_string())?;
    conn.query_row(
        "SELECT name, logo, gstin, address, phone, email, receipt_footer FROM restaurant_info WHERE id = 1",
        [],
        |row| {
            Ok(RestaurantInfo {
                name: row.get(0)?,
                logo: row.get(1)?,
                gstin: row.get(2)?,
                address: row.get(3)?,
                phone: row.get(4)?,
                email: row.get(5)?,
                receipt_footer: row.get(6)?,
            })
        },
    )
    .map_err(|e| e.to_string())
}

#[tauri::command]
fn update_restaurant_info(
    name: String,
    logo: Option<String>,
    gstin: Option<String>,
    address: Option<String>,
    phone: Option<String>,
    email: Option<String>,
    receipt_footer: Option<String>,
    state: tauri::State<'_, db::DbPathState>,
) -> Result<(), String> {
    let conn = rusqlite::Connection::open(&state.path).map_err(|e| e.to_string())?;
    conn.execute(
        "UPDATE restaurant_info SET name = ?1, logo = ?2, gstin = ?3, address = ?4, phone = ?5, email = ?6, receipt_footer = ?7 WHERE id = 1",
        params![name, logo, gstin, address, phone, email, receipt_footer],
    )
    .map_err(|e| e.to_string())?;
    Ok(())
}

#[tauri::command]
fn get_categories(state: tauri::State<'_, db::DbPathState>) -> Result<Vec<Category>, String> {
    let conn = rusqlite::Connection::open(&state.path).map_err(|e| e.to_string())?;
    let mut stmt = conn
        .prepare("SELECT id, name, description FROM categories ORDER BY name ASC")
        .map_err(|e| e.to_string())?;
    
    let cat_iter = stmt
        .query_map([], |row| {
            Ok(Category {
                id: row.get(0)?,
                name: row.get(1)?,
                description: row.get(2)?,
            })
        })
        .map_err(|e| e.to_string())?;
        
    let mut categories = Vec::new();
    for cat in cat_iter {
        categories.push(cat.map_err(|e| e.to_string())?);
    }
    Ok(categories)
}

#[tauri::command]
fn get_products_by_category(
    category_id: i64,
    state: tauri::State<'_, db::DbPathState>,
) -> Result<Vec<Product>, String> {
    let conn = rusqlite::Connection::open(&state.path).map_err(|e| e.to_string())?;
    let mut stmt = conn
        .prepare("SELECT id, category_id, name, price, gst_rate, image_path, is_available FROM products WHERE category_id = ?1")
        .map_err(|e| e.to_string())?;
    
    let prod_iter = stmt
        .query_map([category_id], |row| {
            let is_available_val: i32 = row.get(6)?;
            Ok(Product {
                id: row.get(0)?,
                category_id: row.get(1)?,
                name: row.get(2)?,
                price: row.get(3)?,
                gst_rate: row.get(4)?,
                image_path: row.get(5)?,
                is_available: is_available_val != 0,
            })
        })
        .map_err(|e| e.to_string())?;
        
    let mut products = Vec::new();
    for prod in prod_iter {
        products.push(prod.map_err(|e| e.to_string())?);
    }
    Ok(products)
}

// Advanced Billing & Holds
#[tauri::command]
fn create_order(
    table_id: Option<i64>,
    customer_id: Option<i64>,
    subtotal: f64,
    tax: f64,
    discount: f64,
    service_charge: f64,
    round_off: f64,
    total: f64,
    status: String,
    payment_mode: Option<String>,
    notes: Option<String>,
    hold_name: Option<String>,
    items: Vec<OrderItemInput>,
    created_at: String,
    state: tauri::State<'_, db::DbPathState>,
) -> Result<i64, String> {
    let mut conn = rusqlite::Connection::open(&state.path).map_err(|e| e.to_string())?;
    let tx = conn.transaction().map_err(|e| e.to_string())?;
    
    let order_id = {
        tx.execute(
            "INSERT INTO orders (table_id, customer_id, subtotal, tax, discount, service_charge, round_off, total, status, payment_mode, notes, hold_name, created_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13)",
            params![table_id, customer_id, subtotal, tax, discount, service_charge, round_off, total, status, payment_mode, notes, hold_name, created_at],
        ).map_err(|e| e.to_string())?;
        tx.last_insert_rowid()
    };
    
    for item in &items {
        tx.execute(
            "INSERT INTO order_items (order_id, product_id, name, quantity, price, gst_rate, notes)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
            params![order_id, item.product_id, item.name, item.quantity, item.price, item.gst_rate, item.notes],
        ).map_err(|e| e.to_string())?;
        
        tx.execute(
            "UPDATE inventory SET stock_qty = MAX(0, stock_qty - ?1) WHERE product_id = ?2",
            params![item.quantity, item.product_id],
        ).map_err(|e| e.to_string())?;
    }
    
    if let Some(tid) = table_id {
        let table_status = if status == "Pending" || status == "Billed" {
            "Occupied"
        } else {
            "Free"
        };
        let current_order = if status == "Pending" || status == "Billed" {
            Some(order_id)
        } else {
            None
        };
        tx.execute(
            "UPDATE tables SET status = ?1, current_order_id = ?2 WHERE id = ?3",
            params![table_status, current_order, tid],
        ).map_err(|e| e.to_string())?;
    }
    
    if status == "Completed" {
        if let Some(cid) = customer_id {
            let pts_gained = (total / 100.0) as i64;
            tx.execute(
                "UPDATE customers SET loyalty_points = loyalty_points + ?1 WHERE id = ?2",
                params![pts_gained, cid],
            ).map_err(|e| e.to_string())?;
        }
    }
    
    if status == "Pending" || status == "Billed" {
        tx.execute(
            "INSERT INTO kot (order_id, status, created_at) VALUES (?1, 'Pending', ?2)",
            params![order_id, created_at],
        ).map_err(|e| e.to_string())?;
        let kot_id = tx.last_insert_rowid();
        
        for item in &items {
            tx.execute(
                "INSERT INTO kot_items (kot_id, product_id, quantity, notes) VALUES (?1, ?2, ?3, ?4)",
                params![kot_id, item.product_id, item.quantity, item.notes],
            ).map_err(|e| e.to_string())?;
        }
    }
    
    tx.commit().map_err(|e| e.to_string())?;
    Ok(order_id)
}

#[tauri::command]
fn update_order(
    order_id: i64,
    table_id: Option<i64>,
    customer_id: Option<i64>,
    subtotal: f64,
    tax: f64,
    discount: f64,
    service_charge: f64,
    round_off: f64,
    total: f64,
    status: String,
    payment_mode: Option<String>,
    notes: Option<String>,
    hold_name: Option<String>,
    items: Vec<OrderItemInput>,
    created_at: String,
    state: tauri::State<'_, db::DbPathState>,
) -> Result<(), String> {
    let mut conn = rusqlite::Connection::open(&state.path).map_err(|e| e.to_string())?;
    let tx = conn.transaction().map_err(|e| e.to_string())?;

    // 1. Fetch old table_id for table state management later
    let old_table_id: Option<i64> = tx.query_row(
        "SELECT table_id FROM orders WHERE id = ?1",
        [order_id],
        |row| row.get(0),
    ).map_err(|e| e.to_string())?;

    // 2. Fetch existing items in this order
    let mut old_items = std::collections::HashMap::new();
    {
        let mut stmt = tx.prepare(
            "SELECT product_id, quantity FROM order_items WHERE order_id = ?1"
        ).map_err(|e| e.to_string())?;
        
        let old_items_iter = stmt.query_map([order_id], |row| {
            Ok((row.get::<_, i64>(0)?, row.get::<_, i64>(1)?))
        }).map_err(|e| e.to_string())?;

        for item in old_items_iter {
            let (product_id, qty) = item.map_err(|e| e.to_string())?;
            old_items.insert(product_id, qty);
        }
    }

    // 3. Determine new additions / changes and update stock and build incremental KOT items list
    let mut kot_additions = Vec::new();
    let mut processed_products = std::collections::HashSet::new();

    for item in &items {
        processed_products.insert(item.product_id);
        if let Some(&old_qty) = old_items.get(&item.product_id) {
            let diff = item.quantity - old_qty;
            if diff > 0 {
                // Quantity increased: send difference to kitchen
                kot_additions.push(OrderItemInput {
                    product_id: item.product_id,
                    name: item.name.clone(),
                    quantity: diff,
                    price: item.price,
                    gst_rate: item.gst_rate,
                    notes: item.notes.clone(),
                });
                // Deduct diff from stock
                tx.execute(
                    "UPDATE inventory SET stock_qty = MAX(0, stock_qty - ?1) WHERE product_id = ?2",
                    params![diff, item.product_id],
                ).map_err(|e| e.to_string())?;
            } else if diff < 0 {
                // Quantity decreased: refund difference to stock
                let refund = -diff;
                tx.execute(
                    "UPDATE inventory SET stock_qty = stock_qty + ?1 WHERE product_id = ?2",
                    params![refund, item.product_id],
                ).map_err(|e| e.to_string())?;
            }
        } else {
            // Brand-new item: send full quantity to kitchen
            kot_additions.push(item.clone());
            // Deduct full quantity from stock
            tx.execute(
                "UPDATE inventory SET stock_qty = MAX(0, stock_qty - ?1) WHERE product_id = ?2",
                params![item.quantity, item.product_id],
            ).map_err(|e| e.to_string())?;
        }
    }

    // 4. Refund stock for any items that were completely removed
    for (&product_id, &old_qty) in &old_items {
        if !processed_products.contains(&product_id) {
            tx.execute(
                "UPDATE inventory SET stock_qty = stock_qty + ?1 WHERE product_id = ?2",
                params![old_qty, product_id],
            ).map_err(|e| e.to_string())?;
        }
    }

    // 5. Delete and replace items in order_items
    tx.execute(
        "DELETE FROM order_items WHERE order_id = ?1",
        [order_id],
    ).map_err(|e| e.to_string())?;

    for item in &items {
        tx.execute(
            "INSERT INTO order_items (order_id, product_id, name, quantity, price, gst_rate, notes)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
            params![order_id, item.product_id, item.name, item.quantity, item.price, item.gst_rate, item.notes],
        ).map_err(|e| e.to_string())?;
    }

    // 6. Update orders table row
    tx.execute(
        "UPDATE orders 
         SET table_id = ?1, customer_id = ?2, subtotal = ?3, tax = ?4, discount = ?5, service_charge = ?6, round_off = ?7, total = ?8, status = ?9, payment_mode = ?10, notes = ?11, hold_name = ?12
         WHERE id = ?13",
        params![table_id, customer_id, subtotal, tax, discount, service_charge, round_off, total, status, payment_mode, notes, hold_name, order_id],
    ).map_err(|e| e.to_string())?;

    // 7. Table Status Management
    // Free the old table if it has changed or the order is finalized
    if let Some(old_tid) = old_table_id {
        if table_id != Some(old_tid) || status == "Completed" || status == "Cancelled" {
            tx.execute(
                "UPDATE tables SET status = 'Free', current_order_id = NULL WHERE id = ?1",
                [old_tid],
            ).map_err(|e| e.to_string())?;
        }
    }
    
    // Set status of new table if active and table exists
    if let Some(tid) = table_id {
        if status == "Pending" || status == "Billed" {
            tx.execute(
                "UPDATE tables SET status = 'Occupied', current_order_id = ?1 WHERE id = ?2",
                params![order_id, tid],
            ).map_err(|e| e.to_string())?;
        }
    }

    // 8. Loyalty points award for completed order
    if status == "Completed" {
        if let Some(cid) = customer_id {
            let pts_gained = (total / 100.0) as i64;
            tx.execute(
                "UPDATE customers SET loyalty_points = loyalty_points + ?1 WHERE id = ?2",
                params![pts_gained, cid],
            ).map_err(|e| e.to_string())?;
        }
    }

    // 9. Generate incremental KOT only if there are new items to cook
    if (status == "Pending" || status == "Billed") && !kot_additions.is_empty() {
        tx.execute(
            "INSERT INTO kot (order_id, status, created_at) VALUES (?1, 'Pending', ?2)",
            params![order_id, created_at],
        ).map_err(|e| e.to_string())?;
        let kot_id = tx.last_insert_rowid();

        for item in &kot_additions {
            tx.execute(
                "INSERT INTO kot_items (kot_id, product_id, quantity, notes) VALUES (?1, ?2, ?3, ?4)",
                params![kot_id, item.product_id, item.quantity, item.notes],
            ).map_err(|e| e.to_string())?;
        }
    }

    tx.commit().map_err(|e| e.to_string())?;
    Ok(())
}

#[tauri::command]
fn get_order(order_id: i64, state: tauri::State<'_, db::DbPathState>) -> Result<OrderOutput, String> {
    let conn = rusqlite::Connection::open(&state.path).map_err(|e| e.to_string())?;
    
    let header = conn.query_row(
        "SELECT o.id, o.table_id, t.name, o.customer_id, c.name, o.subtotal, o.tax, o.discount, o.service_charge, o.round_off, o.total, o.status, o.payment_mode, o.notes, o.hold_name, o.created_at 
         FROM orders o 
         LEFT JOIN tables t ON o.table_id = t.id 
         LEFT JOIN customers c ON o.customer_id = c.id
         WHERE o.id = ?1",
        [order_id],
        |row| {
            Ok(OrderHeader {
                id: row.get(0)?,
                table_id: row.get(1)?,
                table_name: row.get(2)?,
                customer_id: row.get(3)?,
                customer_name: row.get(4)?,
                subtotal: row.get(5)?,
                tax: row.get(6)?,
                discount: row.get(7)?,
                service_charge: row.get(8)?,
                round_off: row.get(9)?,
                total: row.get(10)?,
                status: row.get(11)?,
                payment_mode: row.get(12)?,
                notes: row.get(13)?,
                hold_name: row.get(14)?,
                created_at: row.get(15)?,
            })
        },
    ).map_err(|e| e.to_string())?;
    
    let mut stmt = conn.prepare("SELECT id, product_id, name, quantity, price, gst_rate, notes FROM order_items WHERE order_id = ?1").map_err(|e| e.to_string())?;
    let items_iter = stmt.query_map([order_id], |row| {
        Ok(OrderItemOutput {
            id: row.get(0)?,
            product_id: row.get(1)?,
            name: row.get(2)?,
            quantity: row.get(3)?,
            price: row.get(4)?,
            gst_rate: row.get(5)?,
            notes: row.get(6)?,
        })
    }).map_err(|e| e.to_string())?;
    
    let mut items = Vec::new();
    for it in items_iter {
        items.push(it.map_err(|e| e.to_string())?);
    }
    
    Ok(OrderOutput { header, items })
}

#[tauri::command]
fn get_active_orders(state: tauri::State<'_, db::DbPathState>) -> Result<Vec<OrderHeader>, String> {
    let conn = rusqlite::Connection::open(&state.path).map_err(|e| e.to_string())?;
    let mut stmt = conn.prepare(
        "SELECT o.id, o.table_id, t.name, o.customer_id, c.name, o.subtotal, o.tax, o.discount, o.service_charge, o.round_off, o.total, o.status, o.payment_mode, o.notes, o.hold_name, o.created_at 
         FROM orders o 
         LEFT JOIN tables t ON o.table_id = t.id 
         LEFT JOIN customers c ON o.customer_id = c.id
         WHERE o.status IN ('Pending', 'Billed')
         ORDER BY o.id DESC"
    ).map_err(|e| e.to_string())?;
    
    let order_iter = stmt.query_map([], |row| {
        Ok(OrderHeader {
            id: row.get(0)?,
            table_id: row.get(1)?,
            table_name: row.get(2)?,
            customer_id: row.get(3)?,
            customer_name: row.get(4)?,
            subtotal: row.get(5)?,
            tax: row.get(6)?,
            discount: row.get(7)?,
            service_charge: row.get(8)?,
            round_off: row.get(9)?,
            total: row.get(10)?,
            status: row.get(11)?,
            payment_mode: row.get(12)?,
            notes: row.get(13)?,
            hold_name: row.get(14)?,
            created_at: row.get(15)?,
        })
    }).map_err(|e| e.to_string())?;
    
    let mut orders = Vec::new();
    for ord in order_iter {
        orders.push(ord.map_err(|e| e.to_string())?);
    }
    Ok(orders)
}

#[tauri::command]
fn complete_payment(
    order_id: i64,
    payment_mode: String,
    state: tauri::State<'_, db::DbPathState>,
) -> Result<(), String> {
    let mut conn = rusqlite::Connection::open(&state.path).map_err(|e| e.to_string())?;
    let tx = conn.transaction().map_err(|e| e.to_string())?;
    
    // Retrieve table id & customer info
    let (table_id, customer_id, total): (Option<i64>, Option<i64>, f64) = tx.query_row(
        "SELECT table_id, customer_id, total FROM orders WHERE id = ?1",
        [order_id],
        |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?)),
    ).map_err(|e| e.to_string())?;
    
    // Complete order
    tx.execute(
        "UPDATE orders SET status = 'Completed', payment_mode = ?1 WHERE id = ?2",
        params![payment_mode, order_id],
    ).map_err(|e| e.to_string())?;
    
    // Free table
    if let Some(tid) = table_id {
        tx.execute(
            "UPDATE tables SET status = 'Free', current_order_id = NULL WHERE id = ?1",
            [tid],
        ).map_err(|e| e.to_string())?;
    }
    
    // Award loyalty points (1 point per ₹100 spent)
    if let Some(cid) = customer_id {
        let points = (total / 100.0) as i64;
        tx.execute(
            "UPDATE customers SET loyalty_points = loyalty_points + ?1 WHERE id = ?2",
            params![points, cid],
        ).map_err(|e| e.to_string())?;
    }
    
    tx.commit().map_err(|e| e.to_string())?;
    Ok(())
}

#[tauri::command]
fn cancel_order(order_id: i64, state: tauri::State<'_, db::DbPathState>) -> Result<(), String> {
    let mut conn = rusqlite::Connection::open(&state.path).map_err(|e| e.to_string())?;
    let tx = conn.transaction().map_err(|e| e.to_string())?;
    
    let table_id: Option<i64> = tx.query_row(
        "SELECT table_id FROM orders WHERE id = ?1",
        [order_id],
        |row| row.get(0),
    ).map_err(|e| e.to_string())?;
    
    // Refund stock count
    {
        let mut stmt = tx.prepare("SELECT product_id, quantity FROM order_items WHERE order_id = ?1").map_err(|e| e.to_string())?;
        let items_iter = stmt.query_map([order_id], |row| {
            Ok((row.get::<_, i64>(0)?, row.get::<_, i64>(1)?))
        }).map_err(|e| e.to_string())?;
        
        for it in items_iter {
            let (pid, qty) = it.map_err(|e| e.to_string())?;
            tx.execute(
                "UPDATE inventory SET stock_qty = stock_qty + ?1 WHERE product_id = ?2",
                params![qty, pid],
            ).map_err(|e| e.to_string())?;
        }
    }
    
    // Set status to Cancelled
    tx.execute(
        "UPDATE orders SET status = 'Cancelled' WHERE id = ?1",
        [order_id],
    ).map_err(|e| e.to_string())?;
    
    // Free table
    if let Some(tid) = table_id {
        tx.execute(
            "UPDATE tables SET status = 'Free', current_order_id = NULL WHERE id = ?1",
            [tid],
        ).map_err(|e| e.to_string())?;
    }
    
    // Delete active KOTs for this order
    tx.execute(
        "DELETE FROM kot WHERE order_id = ?1",
        [order_id],
    ).map_err(|e| e.to_string())?;
    
    tx.commit().map_err(|e| e.to_string())?;
    Ok(())
}

// Table Management
#[tauri::command]
fn get_tables(state: tauri::State<'_, db::DbPathState>) -> Result<Vec<TableDetails>, String> {
    let conn = rusqlite::Connection::open(&state.path).map_err(|e| e.to_string())?;
    let mut stmt = conn.prepare(
        "SELECT t.id, t.name, t.status, t.merged_into, t.current_order_id, o.total, o.hold_name
         FROM tables t
         LEFT JOIN orders o ON t.current_order_id = o.id
         ORDER BY t.id ASC"
    ).map_err(|e| e.to_string())?;
    
    let table_iter = stmt.query_map([], |row| {
        Ok(TableDetails {
            id: row.get(0)?,
            name: row.get(1)?,
            status: row.get(2)?,
            merged_into: row.get(3)?,
            current_order_id: row.get(4)?,
            current_order_total: row.get(5)?,
            current_order_hold_name: row.get(6)?,
        })
    }).map_err(|e| e.to_string())?;
    
    let mut tables = Vec::new();
    for tb in table_iter {
        tables.push(tb.map_err(|e| e.to_string())?);
    }
    Ok(tables)
}

#[tauri::command]
fn transfer_table(
    from_table_id: i64,
    to_table_id: i64,
    state: tauri::State<'_, db::DbPathState>,
) -> Result<(), String> {
    let mut conn = rusqlite::Connection::open(&state.path).map_err(|e| e.to_string())?;
    let tx = conn.transaction().map_err(|e| e.to_string())?;
    
    // Get active order id
    let order_id: Option<i64> = tx.query_row(
        "SELECT current_order_id FROM tables WHERE id = ?1",
        [from_table_id],
        |row| row.get(0),
    ).map_err(|e| e.to_string())?;
    
    let oid = order_id.ok_or_else(|| "No active order on source table".to_string())?;
    
    // Verify target is empty
    let target_status: String = tx.query_row(
        "SELECT status FROM tables WHERE id = ?1",
        [to_table_id],
        |row| row.get(0),
    ).map_err(|e| e.to_string())?;
    
    if target_status != "Free" {
        return Err("Target table is not free".to_string());
    }
    
    // Transfer order references
    tx.execute(
        "UPDATE orders SET table_id = ?1 WHERE id = ?2",
        params![to_table_id, oid],
    ).map_err(|e| e.to_string())?;
    
    // Reset source table
    tx.execute(
        "UPDATE tables SET status = 'Free', current_order_id = NULL WHERE id = ?1",
        [from_table_id],
    ).map_err(|e| e.to_string())?;
    
    // Set target table
    tx.execute(
        "UPDATE tables SET status = 'Occupied', current_order_id = ?1 WHERE id = ?2",
        params![oid, to_table_id],
    ).map_err(|e| e.to_string())?;
    
    tx.commit().map_err(|e| e.to_string())?;
    Ok(())
}

#[tauri::command]
fn merge_tables(
    source_table_id: i64,
    target_table_id: i64,
    state: tauri::State<'_, db::DbPathState>,
) -> Result<(), String> {
    let conn = rusqlite::Connection::open(&state.path).map_err(|e| e.to_string())?;
    conn.execute(
        "UPDATE tables SET merged_into = ?1, status = 'Free', current_order_id = NULL WHERE id = ?2",
        params![target_table_id, source_table_id],
    ).map_err(|e| e.to_string())?;
    Ok(())
}

// Kitchen Order Tickets (KOT)
#[tauri::command]
fn get_active_kots(state: tauri::State<'_, db::DbPathState>) -> Result<Vec<KotOutput>, String> {
    let conn = rusqlite::Connection::open(&state.path).map_err(|e| e.to_string())?;
    
    let mut stmt = conn.prepare(
        "SELECT k.id, k.order_id, t.name, k.status, k.created_at 
         FROM kot k
         JOIN orders o ON k.order_id = o.id
         LEFT JOIN tables t ON o.table_id = t.id
         WHERE k.status IN ('Pending', 'Preparing', 'Ready')
         ORDER BY k.id ASC"
    ).map_err(|e| e.to_string())?;
    
    let kot_iter = stmt.query_map([], |row| {
        Ok((
            row.get::<_, i64>(0)?,
            row.get::<_, i64>(1)?,
            row.get::<_, Option<String>>(2)?,
            row.get::<_, String>(3)?,
            row.get::<_, String>(4)?,
        ))
    }).map_err(|e| e.to_string())?;
    
    let mut kots = Vec::new();
    for kt in kot_iter {
        let (kot_id, order_id, table_name, status, created_at) = kt.map_err(|e| e.to_string())?;
        
        // Fetch KOT items
        let mut item_stmt = conn.prepare(
            "SELECT ki.id, ki.product_id, p.name, ki.quantity, ki.notes 
             FROM kot_items ki
             JOIN products p ON ki.product_id = p.id
             WHERE ki.kot_id = ?1"
        ).map_err(|e| e.to_string())?;
        
        let items_iter = item_stmt.query_map([kot_id], |row| {
            Ok(KotItemOutput {
                id: row.get(0)?,
                product_id: row.get(1)?,
                product_name: row.get(2)?,
                quantity: row.get(3)?,
                notes: row.get(4)?,
            })
        }).map_err(|e| e.to_string())?;
        
        let mut items = Vec::new();
        for it in items_iter {
            items.push(it.map_err(|e| e.to_string())?);
        }
        
        kots.push(KotOutput {
            id: kot_id,
            order_id,
            table_name,
            status,
            created_at,
            items,
        });
    }
    
    Ok(kots)
}

#[tauri::command]
fn update_kot_status(
    kot_id: i64,
    status: String,
    state: tauri::State<'_, db::DbPathState>,
) -> Result<(), String> {
    let conn = rusqlite::Connection::open(&state.path).map_err(|e| e.to_string())?;
    conn.execute(
        "UPDATE kot SET status = ?1 WHERE id = ?2",
        params![status, kot_id],
    ).map_err(|e| e.to_string())?;
    Ok(())
}

// Inventory Management
#[tauri::command]
fn get_inventory(state: tauri::State<'_, db::DbPathState>) -> Result<Vec<InventoryOutput>, String> {
    let conn = rusqlite::Connection::open(&state.path).map_err(|e| e.to_string())?;
    let mut stmt = conn.prepare(
        "SELECT i.product_id, p.name, c.name, p.price, i.stock_qty, i.low_stock_threshold
         FROM inventory i
         JOIN products p ON i.product_id = p.id
         JOIN categories c ON p.category_id = c.id
         ORDER BY p.name ASC"
    ).map_err(|e| e.to_string())?;
    
    let inv_iter = stmt.query_map([], |row| {
        Ok(InventoryOutput {
            product_id: row.get(0)?,
            product_name: row.get(1)?,
            category_name: row.get(2)?,
            price: row.get(3)?,
            stock_qty: row.get(4)?,
            low_stock_threshold: row.get(5)?,
        })
    }).map_err(|e| e.to_string())?;
    
    let mut inventory = Vec::new();
    for inv in inv_iter {
        inventory.push(inv.map_err(|e| e.to_string())?);
    }
    Ok(inventory)
}

#[tauri::command]
fn update_stock(
    product_id: i64,
    change_qty: i64,
    state: tauri::State<'_, db::DbPathState>,
) -> Result<(), String> {
    let conn = rusqlite::Connection::open(&state.path).map_err(|e| e.to_string())?;
    conn.execute(
        "UPDATE inventory SET stock_qty = MAX(0, stock_qty + ?1) WHERE product_id = ?2",
        params![change_qty, product_id],
    ).map_err(|e| e.to_string())?;
    Ok(())
}

#[tauri::command]
fn add_purchase(
    product_id: i64,
    quantity: i64,
    supplier: Option<String>,
    unit_price: f64,
    date: String,
    state: tauri::State<'_, db::DbPathState>,
) -> Result<(), String> {
    let mut conn = rusqlite::Connection::open(&state.path).map_err(|e| e.to_string())?;
    let tx = conn.transaction().map_err(|e| e.to_string())?;
    
    tx.execute(
        "INSERT INTO purchase_history (product_id, quantity, supplier, unit_price, date) VALUES (?1, ?2, ?3, ?4, ?5)",
        params![product_id, quantity, supplier, unit_price, date],
    ).map_err(|e| e.to_string())?;
    
    tx.execute(
        "UPDATE inventory SET stock_qty = stock_qty + ?1 WHERE product_id = ?2",
        params![quantity, product_id],
    ).map_err(|e| e.to_string())?;
    
    tx.commit().map_err(|e| e.to_string())?;
    Ok(())
}

// Menu CRUD Editor
#[tauri::command]
fn upsert_category(
    id: Option<i64>,
    name: String,
    description: Option<String>,
    state: tauri::State<'_, db::DbPathState>,
) -> Result<(), String> {
    let conn = rusqlite::Connection::open(&state.path).map_err(|e| e.to_string())?;
    if let Some(cat_id) = id {
        conn.execute(
            "UPDATE categories SET name = ?1, description = ?2 WHERE id = ?3",
            params![name, description, cat_id],
        ).map_err(|e| e.to_string())?;
    } else {
        conn.execute(
            "INSERT INTO categories (name, description) VALUES (?1, ?2)",
            params![name, description],
        ).map_err(|e| e.to_string())?;
    }
    Ok(())
}

#[tauri::command]
fn upsert_product(
    id: Option<i64>,
    category_id: i64,
    name: String,
    price: f64,
    gst_rate: f64,
    is_available: bool,
    state: tauri::State<'_, db::DbPathState>,
) -> Result<(), String> {
    let conn = rusqlite::Connection::open(&state.path).map_err(|e| e.to_string())?;
    let avail_val = if is_available { 1 } else { 0 };
    
    if let Some(prod_id) = id {
        conn.execute(
            "UPDATE products SET category_id = ?1, name = ?2, price = ?3, gst_rate = ?4, is_available = ?5 WHERE id = ?6",
            params![category_id, name, price, gst_rate, avail_val, prod_id],
        ).map_err(|e| e.to_string())?;
    } else {
        let mut conn_mut = conn;
        let tx = conn_mut.transaction().map_err(|e| e.to_string())?;
        
        tx.execute(
            "INSERT INTO products (category_id, name, price, gst_rate, is_available) VALUES (?1, ?2, ?3, ?4, ?5)",
            params![category_id, name, price, gst_rate, avail_val],
        ).map_err(|e| e.to_string())?;
        
        let new_prod_id = tx.last_insert_rowid();
        
        // Add to inventory table
        tx.execute(
            "INSERT INTO inventory (product_id, stock_qty, low_stock_threshold) VALUES (?1, 0, 5)",
            [new_prod_id],
        ).map_err(|e| e.to_string())?;
        
        tx.commit().map_err(|e| e.to_string())?;
    }
    Ok(())
}

// Customers Profile Management
#[tauri::command]
fn get_customers(state: tauri::State<'_, db::DbPathState>) -> Result<Vec<CustomerDetails>, String> {
    let conn = rusqlite::Connection::open(&state.path).map_err(|e| e.to_string())?;
    let mut stmt = conn.prepare("SELECT id, name, phone, email, loyalty_points FROM customers ORDER BY name ASC").map_err(|e| e.to_string())?;
    
    let cust_iter = stmt.query_map([], |row| {
        Ok(CustomerDetails {
            id: row.get(0)?,
            name: row.get(1)?,
            phone: row.get(2)?,
            email: row.get(3)?,
            loyalty_points: row.get(4)?,
        })
    }).map_err(|e| e.to_string())?;
    
    let mut customers = Vec::new();
    for cust in cust_iter {
        customers.push(cust.map_err(|e| e.to_string())?);
    }
    Ok(customers)
}

#[tauri::command]
fn upsert_customer(
    id: Option<i64>,
    name: String,
    phone: String,
    email: Option<String>,
    loyalty_points: Option<i64>,
    state: tauri::State<'_, db::DbPathState>,
) -> Result<i64, String> {
    let conn = rusqlite::Connection::open(&state.path).map_err(|e| e.to_string())?;
    if let Some(cust_id) = id {
        let pts = loyalty_points.unwrap_or(0);
        conn.execute(
            "UPDATE customers SET name = ?1, phone = ?2, email = ?3, loyalty_points = ?4 WHERE id = ?5",
            params![name, phone, email, pts, cust_id],
        ).map_err(|e| e.to_string())?;
        Ok(cust_id)
    } else {
        conn.execute(
            "INSERT INTO customers (name, phone, email, loyalty_points) VALUES (?1, ?2, ?3, 0)",
            params![name, phone, email],
        ).map_err(|e| e.to_string())?;
        Ok(conn.last_insert_rowid())
    }
}

// Sales Report
#[tauri::command]
fn get_sales_report(
    start_date: String,
    end_date: String,
    state: tauri::State<'_, db::DbPathState>,
) -> Result<SalesReportOutput, String> {
    let conn = rusqlite::Connection::open(&state.path).map_err(|e| e.to_string())?;
    
    let mut stmt = conn.prepare(
        "SELECT COALESCE(SUM(total), 0.0), COALESCE(SUM(tax), 0.0), COUNT(id)
         FROM orders 
         WHERE status = 'Completed' AND date(created_at) >= date(?1) AND date(created_at) <= date(?2)"
    ).map_err(|e| e.to_string())?;
    
    let (total_sales, total_tax, order_count): (f64, f64, i64) = stmt.query_row([start_date, end_date], |row| {
        Ok((row.get(0)?, row.get(1)?, row.get(2)?))
    }).map_err(|e| e.to_string())?;
    
    let average_ticket = if order_count > 0 {
        total_sales / (order_count as f64)
    } else {
        0.0
    };
    
    Ok(SalesReportOutput {
        total_sales,
        total_tax,
        order_count,
        average_ticket,
    })
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ProductSalesReport {
    pub name: String,
    pub category_name: String,
    pub quantity: i64,
    pub total_sales: f64,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct PaymentSummaryReport {
    pub cash_sales: f64,
    pub upi_sales: f64,
    pub card_sales: f64,
    pub mixed_sales: f64,
}

#[tauri::command]
fn backup_db(target_path: String, state: tauri::State<'_, db::DbPathState>) -> Result<(), String> {
    let source = &state.path;
    let dest = std::path::Path::new(&target_path);
    
    let dest_file = if dest.is_dir() {
        dest.join("mealdesk_backup.db")
    } else {
        dest.to_path_buf()
    };
    
    std::fs::copy(source, &dest_file)
        .map_err(|e| format!("Failed to create backup: {}", e))?;
    Ok(())
}

#[tauri::command]
fn restore_db(source_path: String, state: tauri::State<'_, db::DbPathState>) -> Result<(), String> {
    let source = std::path::Path::new(&source_path);
    let dest = &state.path;
    
    if !source.exists() {
        return Err("Source backup file does not exist".to_string());
    }
    
    std::fs::copy(source, dest)
        .map_err(|e| format!("Failed to restore database: {}", e))?;
    Ok(())
}

#[tauri::command]
fn get_completed_orders(state: tauri::State<'_, db::DbPathState>) -> Result<Vec<OrderHeader>, String> {
    let conn = rusqlite::Connection::open(&state.path).map_err(|e| e.to_string())?;
    let mut stmt = conn.prepare(
        "SELECT o.id, o.table_id, t.name, o.customer_id, c.name, o.subtotal, o.tax, o.discount, o.service_charge, o.round_off, o.total, o.status, o.payment_mode, o.notes, o.hold_name, o.created_at 
         FROM orders o 
         LEFT JOIN tables t ON o.table_id = t.id 
         LEFT JOIN customers c ON o.customer_id = c.id
         WHERE o.status = 'Completed'
         ORDER BY o.id DESC"
    ).map_err(|e| e.to_string())?;
    
    let order_iter = stmt.query_map([], |row| {
        Ok(OrderHeader {
            id: row.get(0)?,
            table_id: row.get(1)?,
            table_name: row.get(2)?,
            customer_id: row.get(3)?,
            customer_name: row.get(4)?,
            subtotal: row.get(5)?,
            tax: row.get(6)?,
            discount: row.get(7)?,
            service_charge: row.get(8)?,
            round_off: row.get(9)?,
            total: row.get(10)?,
            status: row.get(11)?,
            payment_mode: row.get(12)?,
            notes: row.get(13)?,
            hold_name: row.get(14)?,
            created_at: row.get(15)?,
        })
    }).map_err(|e| e.to_string())?;
    
    let mut orders = Vec::new();
    for ord in order_iter {
        orders.push(ord.map_err(|e| e.to_string())?);
    }
    Ok(orders)
}

#[tauri::command]
fn get_customer_orders(customer_id: i64, state: tauri::State<'_, db::DbPathState>) -> Result<Vec<OrderHeader>, String> {
    let conn = rusqlite::Connection::open(&state.path).map_err(|e| e.to_string())?;
    let mut stmt = conn.prepare(
        "SELECT o.id, o.table_id, t.name, o.customer_id, c.name, o.subtotal, o.tax, o.discount, o.service_charge, o.round_off, o.total, o.status, o.payment_mode, o.notes, o.hold_name, o.created_at 
         FROM orders o 
         LEFT JOIN tables t ON o.table_id = t.id 
         LEFT JOIN customers c ON o.customer_id = c.id
         WHERE o.customer_id = ?1 AND o.status = 'Completed'
         ORDER BY o.id DESC"
    ).map_err(|e| e.to_string())?;
    
    let order_iter = stmt.query_map([customer_id], |row| {
        Ok(OrderHeader {
            id: row.get(0)?,
            table_id: row.get(1)?,
            table_name: row.get(2)?,
            customer_id: row.get(3)?,
            customer_name: row.get(4)?,
            subtotal: row.get(5)?,
            tax: row.get(6)?,
            discount: row.get(7)?,
            service_charge: row.get(8)?,
            round_off: row.get(9)?,
            total: row.get(10)?,
            status: row.get(11)?,
            payment_mode: row.get(12)?,
            notes: row.get(13)?,
            hold_name: row.get(14)?,
            created_at: row.get(15)?,
        })
    }).map_err(|e| e.to_string())?;
    
    let mut orders = Vec::new();
    for ord in order_iter {
        orders.push(ord.map_err(|e| e.to_string())?);
    }
    Ok(orders)
}

#[tauri::command]
fn get_product_sales_report(
    start_date: String,
    end_date: String,
    state: tauri::State<'_, db::DbPathState>,
) -> Result<Vec<ProductSalesReport>, String> {
    let conn = rusqlite::Connection::open(&state.path).map_err(|e| e.to_string())?;
    let mut stmt = conn.prepare(
        "SELECT oi.name, c.name, SUM(oi.quantity), SUM(oi.quantity * oi.price)
         FROM order_items oi
         JOIN orders o ON oi.order_id = o.id
         JOIN products p ON oi.product_id = p.id
         JOIN categories c ON p.category_id = c.id
         WHERE o.status = 'Completed' AND date(o.created_at) >= date(?1) AND date(o.created_at) <= date(?2)
         GROUP BY oi.product_id
         ORDER BY SUM(oi.quantity * oi.price) DESC"
    ).map_err(|e| e.to_string())?;
    
    let report_iter = stmt.query_map([start_date, end_date], |row| {
        Ok(ProductSalesReport {
            name: row.get(0)?,
            category_name: row.get(1)?,
            quantity: row.get(2)?,
            total_sales: row.get(3)?,
        })
    }).map_err(|e| e.to_string())?;
    
    let mut report = Vec::new();
    for rep in report_iter {
        report.push(rep.map_err(|e| e.to_string())?);
    }
    Ok(report)
}

#[tauri::command]
fn get_payment_mode_summary(
    start_date: String,
    end_date: String,
    state: tauri::State<'_, db::DbPathState>,
) -> Result<PaymentSummaryReport, String> {
    let conn = rusqlite::Connection::open(&state.path).map_err(|e| e.to_string())?;
    
    let cash: f64 = conn.query_row(
        "SELECT COALESCE(SUM(total), 0.0) FROM orders WHERE status = 'Completed' AND payment_mode = 'Cash' AND date(created_at) >= date(?1) AND date(created_at) <= date(?2)",
        [&start_date, &end_date],
        |row| row.get(0),
    ).unwrap_or(0.0);
    
    let upi: f64 = conn.query_row(
        "SELECT COALESCE(SUM(total), 0.0) FROM orders WHERE status = 'Completed' AND payment_mode = 'UPI' AND date(created_at) >= date(?1) AND date(created_at) <= date(?2)",
        [&start_date, &end_date],
        |row| row.get(0),
    ).unwrap_or(0.0);
    
    let card: f64 = conn.query_row(
        "SELECT COALESCE(SUM(total), 0.0) FROM orders WHERE status = 'Completed' AND payment_mode = 'Card' AND date(created_at) >= date(?1) AND date(created_at) <= date(?2)",
        [&start_date, &end_date],
        |row| row.get(0),
    ).unwrap_or(0.0);
    
    let mixed: f64 = conn.query_row(
        "SELECT COALESCE(SUM(total), 0.0) FROM orders WHERE status = 'Completed' AND payment_mode = 'Mixed' AND date(created_at) >= date(?1) AND date(created_at) <= date(?2)",
        [&start_date, &end_date],
        |row| row.get(0),
    ).unwrap_or(0.0);
    
    Ok(PaymentSummaryReport {
        cash_sales: cash,
        upi_sales: upi,
        card_sales: card,
        mixed_sales: mixed,
    })
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .setup(|app| {
            let db_path = db::init_db(app.handle())?;
            app.manage(db::DbPathState { path: db_path });
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            greet,
            login,
            get_restaurant_info,
            update_restaurant_info,
            get_categories,
            get_products_by_category,
            create_order,
            update_order,
            get_order,
            get_active_orders,
            complete_payment,
            cancel_order,
            get_tables,
            transfer_table,
            merge_tables,
            get_active_kots,
            update_kot_status,
            get_inventory,
            update_stock,
            add_purchase,
            upsert_category,
            upsert_product,
            get_customers,
            upsert_customer,
            get_sales_report,
            backup_db,
            restore_db,
            get_completed_orders,
            get_customer_orders,
            get_product_sales_report,
            get_payment_mode_summary
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
