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
    pub id: Option<i64>,
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
    pub cancelled_by: Option<String>,
    pub cancelled_at: Option<String>,
    pub cancel_reason: Option<String>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct OrderItemOutput {
    pub id: i64,
    pub product_id: i64,
    pub name: String,
    pub quantity: i64,
    pub cancelled_quantity: i64,
    pub price: f64,
    pub gst_rate: f64,
    pub notes: Option<String>,
    pub kot_id: Option<i64>,
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
    pub print_count: i32,
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

fn get_db_timestamp(conn: &rusqlite::Connection) -> String {
    conn.query_row("SELECT strftime('%Y-%m-%dT%H:%M:%SZ', 'now')", [], |row| row.get(0)).unwrap_or_else(|_| "unknown".to_string())
}

fn log_audit(
    conn: &rusqlite::Connection,
    username: &str,
    action: &str,
    target_type: &str,
    target_id: Option<i64>,
    details: Option<&str>,
) -> Result<(), String> {
    let now = get_db_timestamp(conn);
    conn.execute(
        "INSERT INTO audit_logs (username, action, target_type, target_id, details, created_at)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
        params![username, action, target_type, target_id, details, now],
    ).map_err(|e| format!("Failed to write audit log: {}", e))?;
    Ok(())
}

#[tauri::command]
fn verify_credentials(
    username: String,
    password: Option<String>,
    required_roles: Vec<String>,
    state: tauri::State<'_, db::DbPathState>,
) -> Result<bool, String> {
    let conn = rusqlite::Connection::open(&state.path).map_err(|e| e.to_string())?;
    let password = password.unwrap_or_default();
    let hashed = db::hash_password(&password);
    
    let mut stmt = conn
        .prepare("SELECT role FROM users WHERE username = ?1 AND password_hash = ?2")
        .map_err(|e| e.to_string())?;
    
    let role: Result<String, _> = stmt.query_row([username, hashed], |row| row.get(0));
    match role {
        Ok(r) => Ok(required_roles.contains(&r)),
        Err(_) => Ok(false),
    }
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
    username: String,
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
    
    log_audit(&tx, &username, "create_order", "orders", Some(order_id), Some(&format!("Created order in status: {}", status)))?;
    
    let mut inserted_items = Vec::new();
    for item in &items {
        tx.execute(
            "INSERT INTO order_items (order_id, product_id, name, quantity, price, gst_rate, notes, kot_id)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, NULL)",
            params![order_id, item.product_id, item.name, item.quantity, item.price, item.gst_rate, item.notes],
        ).map_err(|e| e.to_string())?;
        let order_item_id = tx.last_insert_rowid();
        inserted_items.push((order_item_id, item.product_id, item.quantity, item.notes.clone()));
        
        if status != "Draft" {
            tx.execute(
                "UPDATE inventory SET stock_qty = MAX(0, stock_qty - ?1) WHERE product_id = ?2",
                params![item.quantity, item.product_id],
            ).map_err(|e| e.to_string())?;
        }
    }
    
    if let Some(tid) = table_id {
        let table_status = if status == "Draft" || status == "Pending" || status == "Billed" {
            "Occupied"
        } else {
            "Free"
        };
        let current_order = if status == "Draft" || status == "Pending" || status == "Billed" {
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
    
    if status == "Pending" || status == "Billed" || (status == "Completed" && table_id.is_none()) {
        tx.execute(
            "INSERT INTO kot (order_id, status, created_at) VALUES (?1, 'Pending', ?2)",
            params![order_id, created_at],
        ).map_err(|e| e.to_string())?;
        let kot_id = tx.last_insert_rowid();
        
        log_audit(&tx, &username, "create_kot", "kot", Some(kot_id), Some(&format!("Created KOT for order: {}", order_id)))?;

        for (order_item_id, product_id, qty, item_notes) in inserted_items {
            tx.execute(
                "INSERT INTO kot_items (kot_id, product_id, quantity, notes) VALUES (?1, ?2, ?3, ?4)",
                params![kot_id, product_id, qty, item_notes],
            ).map_err(|e| e.to_string())?;
            tx.execute(
                "UPDATE order_items SET kot_id = ?1 WHERE id = ?2",
                params![kot_id, order_item_id],
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
    username: String,
    state: tauri::State<'_, db::DbPathState>,
) -> Result<(), String> {
    let mut conn = rusqlite::Connection::open(&state.path).map_err(|e| e.to_string())?;
    let tx = conn.transaction().map_err(|e| e.to_string())?;

    // 1. Fetch old table_id and old_status for state management later
    let (old_table_id, old_status): (Option<i64>, String) = tx.query_row(
        "SELECT table_id, status FROM orders WHERE id = ?1",
        [order_id],
        |row| Ok((row.get(0)?, row.get(1)?)),
    ).map_err(|e| e.to_string())?;

    log_audit(&tx, &username, "update_order", "orders", Some(order_id), Some(&format!("Updating order. Old status: {}, New status: {}", old_status, status)))?;

    // 2. Fetch existing items in this order
    struct DbItem {
        id: i64,
        product_id: i64,
        quantity: i64,
        price: f64,
        notes: Option<String>,
        kot_id: Option<i64>,
    }

    let db_items = {
        let mut stmt = tx.prepare(
            "SELECT id, product_id, quantity, price, notes, kot_id FROM order_items WHERE order_id = ?1"
        ).map_err(|e| e.to_string())?;
        
        let db_items_iter = stmt.query_map([order_id], |row| {
            Ok(DbItem {
                id: row.get(0)?,
                product_id: row.get(1)?,
                quantity: row.get(2)?,
                price: row.get(3)?,
                notes: row.get(4)?,
                kot_id: row.get(5)?,
            })
        }).map_err(|e| e.to_string())?;

        let mut map = std::collections::HashMap::new();
        for item_res in db_items_iter {
            let item = item_res.map_err(|e| e.to_string())?;
            map.insert(item.id, item);
        }
        map
    };

    // 3. Process incoming items
    let mut processed_incoming_ids = std::collections::HashSet::new();

    for item in &items {
        if let Some(item_id) = item.id {
            processed_incoming_ids.insert(item_id);
            let db_item = db_items.get(&item_id)
                .ok_or_else(|| format!("Order item with ID {} not found in this order", item_id))?;

            if db_item.kot_id.is_some() {
                // Sent to KOT: Immutability constraint verification
                if item.product_id != db_item.product_id || item.quantity != db_item.quantity || item.price != db_item.price {
                    return Err(format!("Sent KOT item (ID {}, Product '{}') is immutable. Modifications must go through the cancellation workflow.", item_id, item.name));
                }
                if item.notes != db_item.notes {
                    tx.execute(
                        "UPDATE order_items SET notes = ?1 WHERE id = ?2",
                        params![item.notes, item_id],
                    ).map_err(|e| e.to_string())?;
                }
            } else {
                // Not sent to KOT (Draft order item)
                if old_status == "Draft" && status != "Draft" && status != "Cancelled" {
                    // Transition: Draft -> Active: Deduct full quantity from stock
                    tx.execute(
                        "UPDATE inventory SET stock_qty = MAX(0, stock_qty - ?1) WHERE product_id = ?2",
                        params![item.quantity, item.product_id],
                    ).map_err(|e| e.to_string())?;
                } else if old_status != "Draft" && status != "Cancelled" {
                    // Transition: Active -> Active: Update quantity and handle stock difference
                    let diff = item.quantity - db_item.quantity;
                    if diff > 0 {
                        tx.execute(
                            "UPDATE inventory SET stock_qty = MAX(0, stock_qty - ?1) WHERE product_id = ?2",
                            params![diff, item.product_id],
                        ).map_err(|e| e.to_string())?;
                    } else if diff < 0 {
                        let refund = -diff;
                        tx.execute(
                            "UPDATE inventory SET stock_qty = stock_qty + ?1 WHERE product_id = ?2",
                            params![refund, item.product_id],
                        ).map_err(|e| e.to_string())?;
                    }
                }
                tx.execute(
                    "UPDATE order_items SET quantity = ?1, notes = ?2, price = ?3 WHERE id = ?4",
                    params![item.quantity, item.notes, item.price, item_id],
                ).map_err(|e| e.to_string())?;
            }
        } else {
            // Brand new order item added
            tx.execute(
                "INSERT INTO order_items (order_id, product_id, name, quantity, price, gst_rate, notes, kot_id)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, NULL)",
                params![order_id, item.product_id, item.name, item.quantity, item.price, item.gst_rate, item.notes],
            ).map_err(|e| e.to_string())?;
            let new_item_id = tx.last_insert_rowid();
            
            log_audit(&tx, &username, "add_item", "order_items", Some(new_item_id), Some(&format!("Added item {} (Qty {}) to order {}", item.name, item.quantity, order_id)))?;

            // If order is already active, deduct stock now
            if old_status != "Draft" && status != "Cancelled" {
                tx.execute(
                    "UPDATE inventory SET stock_qty = MAX(0, stock_qty - ?1) WHERE product_id = ?2",
                    params![item.quantity, item.product_id],
                ).map_err(|e| e.to_string())?;
            }
        }
    }

    // 4. Handle deleted items
    for (&db_item_id, db_item) in &db_items {
        if !processed_incoming_ids.contains(&db_item_id) {
            if db_item.kot_id.is_some() {
                return Err(format!("Cannot delete order item ID {} because it was already sent to the kitchen.", db_item_id));
            }
            
            if old_status != "Draft" {
                tx.execute(
                    "UPDATE inventory SET stock_qty = stock_qty + ?1 WHERE product_id = ?2",
                    params![db_item.quantity, db_item.product_id],
                ).map_err(|e| e.to_string())?;
            }
            
            tx.execute("DELETE FROM order_items WHERE id = ?1", [db_item_id]).map_err(|e| e.to_string())?;
            log_audit(&tx, &username, "delete_item", "order_items", Some(db_item_id), Some(&format!("Deleted unsent item from order {}", order_id)))?;
        }
    }

    // 5. Update orders table row
    tx.execute(
        "UPDATE orders 
         SET table_id = ?1, customer_id = ?2, subtotal = ?3, tax = ?4, discount = ?5, service_charge = ?6, round_off = ?7, total = ?8, status = ?9, payment_mode = ?10, notes = ?11, hold_name = ?12
         WHERE id = ?13",
        params![table_id, customer_id, subtotal, tax, discount, service_charge, round_off, total, status, payment_mode, notes, hold_name, order_id],
    ).map_err(|e| e.to_string())?;

    // 6. KOT Generation for Unsent Items
    if status == "Pending" || status == "Billed" || (status == "Completed" && table_id.is_none()) {
        struct UnsentItem {
            id: i64,
            product_id: i64,
            quantity: i64,
            notes: Option<String>,
        }
        
        let unsent_items = {
            let mut stmt = tx.prepare(
                "SELECT id, product_id, quantity, notes FROM order_items WHERE order_id = ?1 AND kot_id IS NULL"
            ).map_err(|e| e.to_string())?;
            
            let unsent_iter = stmt.query_map([order_id], |row| {
                Ok(UnsentItem {
                    id: row.get(0)?,
                    product_id: row.get(1)?,
                    quantity: row.get(2)?,
                    notes: row.get(3)?,
                })
            }).map_err(|e| e.to_string())?;
            
            let mut list = Vec::new();
            for item in unsent_iter {
                list.push(item.map_err(|e| e.to_string())?);
            }
            list
        };
        
        if !unsent_items.is_empty() {
            tx.execute(
                "INSERT INTO kot (order_id, status, created_at) VALUES (?1, 'Pending', ?2)",
                params![order_id, created_at],
            ).map_err(|e| e.to_string())?;
            let kot_id = tx.last_insert_rowid();
            
            log_audit(&tx, &username, "create_kot", "kot", Some(kot_id), Some(&format!("Created KOT for order: {}", order_id)))?;

            for item in unsent_items {
                tx.execute(
                    "INSERT INTO kot_items (kot_id, product_id, quantity, notes) VALUES (?1, ?2, ?3, ?4)",
                    params![kot_id, item.product_id, item.quantity, item.notes],
                ).map_err(|e| e.to_string())?;
                tx.execute(
                    "UPDATE order_items SET kot_id = ?1 WHERE id = ?2",
                    params![kot_id, item.id],
                ).map_err(|e| e.to_string())?;
            }
        }
    }

    // 7. Table Status Management
    if let Some(old_tid) = old_table_id {
        if table_id != Some(old_tid) || status == "Completed" || status == "Cancelled" {
            tx.execute(
                "UPDATE tables SET status = 'Free', current_order_id = NULL WHERE id = ?1",
                [old_tid],
            ).map_err(|e| e.to_string())?;
        }
    }
    
    if let Some(tid) = table_id {
        if status == "Draft" || status == "Pending" || status == "Billed" {
            tx.execute(
                "UPDATE tables SET status = 'Occupied', current_order_id = ?1 WHERE id = ?2",
                params![order_id, tid],
            ).map_err(|e| e.to_string())?;
        }
    }

    // 8. Loyalty points award
    if status == "Completed" {
        if let Some(cid) = customer_id {
            let pts_gained = (total / 100.0) as i64;
            tx.execute(
                "UPDATE customers SET loyalty_points = loyalty_points + ?1 WHERE id = ?2",
                params![pts_gained, cid],
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
        "SELECT o.id, o.table_id, t.name, o.customer_id, c.name, o.subtotal, o.tax, o.discount, o.service_charge, o.round_off, o.total, o.status, o.payment_mode, o.notes, o.hold_name, o.created_at, o.cancelled_by, o.cancelled_at, o.cancel_reason
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
                cancelled_by: row.get(16)?,
                cancelled_at: row.get(17)?,
                cancel_reason: row.get(18)?,
            })
        },
    ).map_err(|e| e.to_string())?;
    
    let mut stmt = conn.prepare(
        "SELECT oi.id, oi.product_id, oi.name, oi.quantity, 
                COALESCE((SELECT SUM(c.quantity) FROM order_item_cancellations c WHERE c.order_item_id = oi.id), 0) AS cancelled_quantity,
                oi.price, oi.gst_rate, oi.notes, oi.kot_id 
         FROM order_items oi 
         WHERE oi.order_id = ?1"
    ).map_err(|e| e.to_string())?;
    
    let items_iter = stmt.query_map([order_id], |row| {
        Ok(OrderItemOutput {
            id: row.get(0)?,
            product_id: row.get(1)?,
            name: row.get(2)?,
            quantity: row.get(3)?,
            cancelled_quantity: row.get(4)?,
            price: row.get(5)?,
            gst_rate: row.get(6)?,
            notes: row.get(7)?,
            kot_id: row.get(8)?,
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
        "SELECT o.id, o.table_id, t.name, o.customer_id, c.name, o.subtotal, o.tax, o.discount, o.service_charge, o.round_off, o.total, o.status, o.payment_mode, o.notes, o.hold_name, o.created_at, o.cancelled_by, o.cancelled_at, o.cancel_reason
         FROM orders o 
         LEFT JOIN tables t ON o.table_id = t.id 
         LEFT JOIN customers c ON o.customer_id = c.id
         WHERE o.status IN ('Draft', 'Pending', 'Billed')
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
            cancelled_by: row.get(16)?,
            cancelled_at: row.get(17)?,
            cancel_reason: row.get(18)?,
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
    username: String,
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
    
    log_audit(&tx, &username, "complete_payment", "orders", Some(order_id), Some(&format!("Completed payment using mode: {}", payment_mode)))?;
    
    tx.commit().map_err(|e| e.to_string())?;
    Ok(())
}

#[tauri::command]
fn cancel_order(
    order_id: i64,
    cancelled_by: String,
    reason: String,
    state: tauri::State<'_, db::DbPathState>,
) -> Result<(), String> {
    let mut conn = rusqlite::Connection::open(&state.path).map_err(|e| e.to_string())?;
    let tx = conn.transaction().map_err(|e| e.to_string())?;
    
    // Fetch table_id and current status
    let (table_id, current_status): (Option<i64>, String) = tx.query_row(
        "SELECT table_id, status FROM orders WHERE id = ?1",
        [order_id],
        |row| Ok((row.get(0)?, row.get(1)?)),
    ).map_err(|e| e.to_string())?;

    if current_status == "Cancelled" {
        return Err("Order is already cancelled".to_string());
    }
    
    // Refund stock count ONLY if status is NOT 'Draft'
    let mut kot_cancellations = Vec::new();
    if current_status != "Draft" {
        struct ItemDetails {
            product_id: i64,
            quantity: i64,
            cancelled_qty: i64,
            notes: Option<String>,
            kot_id: Option<i64>,
        }
        
        let mut stmt = tx.prepare(
            "SELECT oi.product_id, oi.name, oi.quantity, 
                    COALESCE((SELECT SUM(c.quantity) FROM order_item_cancellations c WHERE c.order_item_id = oi.id), 0) AS cancelled_qty,
                    oi.notes, oi.kot_id
             FROM order_items oi
             WHERE oi.order_id = ?1"
        ).map_err(|e| e.to_string())?;
        
        let items_iter = stmt.query_map([order_id], |row| {
            Ok(ItemDetails {
                product_id: row.get(0)?,
                quantity: row.get(2)?,
                cancelled_qty: row.get(3)?,
                notes: row.get(4)?,
                kot_id: row.get(5)?,
            })
        }).map_err(|e| e.to_string())?;
        
        for item_res in items_iter {
            let item = item_res.map_err(|e| e.to_string())?;
            let effective_qty = item.quantity - item.cancelled_qty;
            if effective_qty > 0 {
                tx.execute(
                    "UPDATE inventory SET stock_qty = stock_qty + ?1 WHERE product_id = ?2",
                    params![effective_qty, item.product_id],
                ).map_err(|e| e.to_string())?;
                
                if item.kot_id.is_some() {
                    kot_cancellations.push((item.product_id, effective_qty, item.notes));
                }
            }
        }
    }
    
    // Set status to Cancelled and record cancellation info
    let now = get_db_timestamp(&tx);
    tx.execute(
        "UPDATE orders SET status = 'Cancelled', cancelled_by = ?1, cancelled_at = ?2, cancel_reason = ?3 WHERE id = ?4",
        params![cancelled_by, now, reason, order_id],
    ).map_err(|e| e.to_string())?;
    
    // Free table
    if let Some(tid) = table_id {
        tx.execute(
            "UPDATE tables SET status = 'Free', current_order_id = NULL WHERE id = ?1",
            [tid],
        ).map_err(|e| e.to_string())?;
    }
    
    // Generate a KOT representing the cancellation if there are items to cancel in kitchen
    if !kot_cancellations.is_empty() {
        tx.execute(
            "INSERT INTO kot (order_id, status, created_at) VALUES (?1, 'Pending', ?2)",
            params![order_id, now],
        ).map_err(|e| e.to_string())?;
        let kot_id = tx.last_insert_rowid();
        
        for (prod_id, qty, notes) in kot_cancellations {
            let cancel_notes = match notes {
                Some(n) => format!("CANCELLED: {} (Reason: {})", n, reason),
                None => format!("CANCELLED (Reason: {})", reason),
            };
            tx.execute(
                "INSERT INTO kot_items (kot_id, product_id, quantity, notes) VALUES (?1, ?2, ?3, ?4)",
                params![kot_id, prod_id, -qty, cancel_notes],
            ).map_err(|e| e.to_string())?;
        }
    }

    log_audit(&tx, &cancelled_by, "cancel_order", "orders", Some(order_id), Some(&format!("Cancelled entire order. Reason: {}", reason)))?;
    
    tx.commit().map_err(|e| e.to_string())?;
    Ok(())
}

#[tauri::command]
fn cancel_order_item(
    order_item_id: i64,
    quantity_to_cancel: i64,
    cancelled_by: String,
    reason: String,
    state: tauri::State<'_, db::DbPathState>,
) -> Result<(), String> {
    let mut conn = rusqlite::Connection::open(&state.path).map_err(|e| e.to_string())?;
    let tx = conn.transaction().map_err(|e| e.to_string())?;
    
    struct ItemInfo {
        order_id: i64,
        product_id: i64,
        name: String,
        quantity: i64,
        notes: Option<String>,
        kot_id: Option<i64>,
    }
    
    let item: ItemInfo = tx.query_row(
        "SELECT order_id, product_id, name, quantity, notes, kot_id FROM order_items WHERE id = ?1",
        [order_item_id],
        |row| Ok(ItemInfo {
            order_id: row.get(0)?,
            product_id: row.get(1)?,
            name: row.get(2)?,
            quantity: row.get(3)?,
            notes: row.get(4)?,
            kot_id: row.get(5)?,
        }),
    ).map_err(|e| e.to_string())?;
    
    if item.kot_id.is_none() {
        return Err("Cannot cancel an item that has not been sent to the kitchen. You can remove it from the cart directly.".to_string());
    }
    
    let already_cancelled: i64 = tx.query_row(
        "SELECT COALESCE(SUM(quantity), 0) FROM order_item_cancellations WHERE order_item_id = ?1",
        [order_item_id],
        |row| row.get(0),
    ).unwrap_or(0);
    
    let effective_qty = item.quantity - already_cancelled;
    if quantity_to_cancel <= 0 || quantity_to_cancel > effective_qty {
        return Err(format!("Invalid quantity to cancel: requested {}, effective is {}", quantity_to_cancel, effective_qty));
    }
    
    let now = get_db_timestamp(&tx);
    tx.execute(
        "INSERT INTO order_item_cancellations (order_item_id, quantity, reason, cancelled_by, created_at)
         VALUES (?1, ?2, ?3, ?4, ?5)",
        params![order_item_id, quantity_to_cancel, reason, cancelled_by, now],
    ).map_err(|e| e.to_string())?;
    
    tx.execute(
        "UPDATE inventory SET stock_qty = stock_qty + ?1 WHERE product_id = ?2",
        params![quantity_to_cancel, item.product_id],
    ).map_err(|e| e.to_string())?;
    
    tx.execute(
        "INSERT INTO kot (order_id, status, created_at) VALUES (?1, 'Pending', ?2)",
        params![item.order_id, now],
    ).map_err(|e| e.to_string())?;
    let new_kot_id = tx.last_insert_rowid();
    
    let cancel_notes = match item.notes {
        Some(n) => format!("CANCELLED: {} (Reason: {})", n, reason),
        None => format!("CANCELLED (Reason: {})", reason),
    };
    tx.execute(
        "INSERT INTO kot_items (kot_id, product_id, quantity, notes) VALUES (?1, ?2, ?3, ?4)",
        params![new_kot_id, item.product_id, -quantity_to_cancel, cancel_notes],
    ).map_err(|e| e.to_string())?;
    
    log_audit(&tx, &cancelled_by, "cancel_item", "order_items", Some(order_item_id), Some(&format!("Cancelled {} x {}. Reason: {}", quantity_to_cancel, item.name, reason)))?;
    
    recalculate_order_totals(&tx, item.order_id)?;
    
    tx.commit().map_err(|e| e.to_string())?;
    Ok(())
}

fn recalculate_order_totals(conn: &rusqlite::Connection, order_id: i64) -> Result<(), String> {
    let (discount, service_charge): (f64, f64) = conn.query_row(
        "SELECT discount, service_charge FROM orders WHERE id = ?1",
        [order_id],
        |row| Ok((row.get(0)?, row.get(1)?)),
    ).map_err(|e| e.to_string())?;
    
    struct ItemTotals {
        quantity: i64,
        cancelled_qty: i64,
        price: f64,
        gst_rate: f64,
    }
    
    let mut stmt = conn.prepare(
        "SELECT oi.quantity, 
                COALESCE((SELECT SUM(c.quantity) FROM order_item_cancellations c WHERE c.order_item_id = oi.id), 0) AS cancelled_qty,
                oi.price, oi.gst_rate
         FROM order_items oi
         WHERE oi.order_id = ?1"
    ).map_err(|e| e.to_string())?;
    
    let items_iter = stmt.query_map([order_id], |row| {
        Ok(ItemTotals {
            quantity: row.get(0)?,
            cancelled_qty: row.get(1)?,
            price: row.get(2)?,
            gst_rate: row.get(3)?,
        })
    }).map_err(|e| e.to_string())?;
    
    let mut subtotal = 0.0;
    let mut total_tax = 0.0;
    
    for item_res in items_iter {
        let item = item_res.map_err(|e| e.to_string())?;
        let effective_qty = item.quantity - item.cancelled_qty;
        if effective_qty > 0 {
            let item_subtotal = item.price * (effective_qty as f64);
            let item_tax = item_subtotal * (item.gst_rate / 100.0);
            subtotal += item_subtotal;
            total_tax += item_tax;
        }
    }
    
    let discount_amount = subtotal * (discount / 100.0);
    let service_amount = subtotal * (service_charge / 100.0);
    let taxable_subtotal = subtotal - discount_amount + service_amount;
    
    let final_tax = if subtotal > 0.0 {
        taxable_subtotal * (total_tax / subtotal)
    } else {
        0.0
    };
    
    let total_raw = taxable_subtotal + final_tax;
    let total_rounded = total_raw.round();
    let round_off = total_rounded - total_raw;
    
    conn.execute(
        "UPDATE orders 
         SET subtotal = ?1, tax = ?2, round_off = ?3, total = ?4 
         WHERE id = ?5",
        params![subtotal, final_tax, round_off, total_rounded, order_id],
    ).map_err(|e| e.to_string())?;
    
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
    username: String,
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
    
    log_audit(&tx, &username, "transfer_table", "tables", Some(from_table_id), Some(&format!("Transferred order {} from table ID {} to ID {}", oid, from_table_id, to_table_id)))?;
    
    tx.commit().map_err(|e| e.to_string())?;
    Ok(())
}

#[tauri::command]
fn merge_tables(
    source_table_id: i64,
    target_table_id: i64,
    username: String,
    state: tauri::State<'_, db::DbPathState>,
) -> Result<(), String> {
    let conn = rusqlite::Connection::open(&state.path).map_err(|e| e.to_string())?;
    conn.execute(
        "UPDATE tables SET merged_into = ?1, status = 'Free', current_order_id = NULL WHERE id = ?2",
        params![target_table_id, source_table_id],
    ).map_err(|e| e.to_string())?;
    
    log_audit(&conn, &username, "merge_tables", "tables", Some(source_table_id), Some(&format!("Merged table ID {} into table ID {}", source_table_id, target_table_id)))?;
    Ok(())
}

// Kitchen Order Tickets (KOT)
#[tauri::command]
fn get_active_kots(state: tauri::State<'_, db::DbPathState>) -> Result<Vec<KotOutput>, String> {
    let conn = rusqlite::Connection::open(&state.path).map_err(|e| e.to_string())?;
    
    let mut stmt = conn.prepare(
        "SELECT k.id, k.order_id, t.name, k.status, k.print_count, k.created_at 
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
            row.get::<_, i32>(4)?,
            row.get::<_, String>(5)?,
        ))
    }).map_err(|e| e.to_string())?;
    
    let mut kots = Vec::new();
    for kt in kot_iter {
        let (kot_id, order_id, table_name, status, print_count, created_at) = kt.map_err(|e| e.to_string())?;
        
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
            print_count,
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
    username: String,
    state: tauri::State<'_, db::DbPathState>,
) -> Result<(), String> {
    let mut conn = rusqlite::Connection::open(&state.path).map_err(|e| e.to_string())?;
    let tx = conn.transaction().map_err(|e| e.to_string())?;
    
    let current_status: String = tx.query_row(
        "SELECT status FROM kot WHERE id = ?1",
        [kot_id],
        |row| row.get(0),
    ).map_err(|e| e.to_string())?;
    
    // Validate transition: Pending -> Preparing -> Ready -> Completed
    let is_valid = match (current_status.as_str(), status.as_str()) {
        ("Pending", "Preparing") => true,
        ("Preparing", "Ready") => true,
        ("Ready", "Completed") => true,
        _ => false,
    };
    
    if !is_valid {
        return Err(format!("Invalid KOT status transition from {} to {}", current_status, status));
    }
    
    tx.execute(
        "UPDATE kot SET status = ?1 WHERE id = ?2",
        params![status, kot_id],
    ).map_err(|e| e.to_string())?;
    
    log_audit(&tx, &username, "update_kot_status", "kot", Some(kot_id), Some(&format!("KOT status updated from {} to {}", current_status, status)))?;
    
    tx.commit().map_err(|e| e.to_string())?;
    Ok(())
}

#[tauri::command]
fn get_kot_by_id(kot_id: i64, state: tauri::State<'_, db::DbPathState>) -> Result<KotOutput, String> {
    let conn = rusqlite::Connection::open(&state.path).map_err(|e| e.to_string())?;
    
    let (order_id, table_name, status, print_count, created_at) = conn.query_row(
        "SELECT k.order_id, t.name, k.status, k.print_count, k.created_at 
         FROM kot k
         JOIN orders o ON k.order_id = o.id
         LEFT JOIN tables t ON o.table_id = t.id
         WHERE k.id = ?1",
        params![kot_id],
        |row| {
            Ok((
                row.get::<_, i64>(0)?,
                row.get::<_, Option<String>>(1)?,
                row.get::<_, String>(2)?,
                row.get::<_, i32>(3)?,
                row.get::<_, String>(4)?,
            ))
        }
    ).map_err(|e| e.to_string())?;
    
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
    
    Ok(KotOutput {
        id: kot_id,
        order_id,
        table_name,
        status,
        print_count,
        created_at,
        items,
    })
}

#[tauri::command]
fn get_kots_for_order(order_id: i64, state: tauri::State<'_, db::DbPathState>) -> Result<Vec<KotOutput>, String> {
    let conn = rusqlite::Connection::open(&state.path).map_err(|e| e.to_string())?;
    
    let mut stmt = conn.prepare(
        "SELECT k.id, t.name, k.status, k.print_count, k.created_at 
         FROM kot k
         JOIN orders o ON k.order_id = o.id
         LEFT JOIN tables t ON o.table_id = t.id
         WHERE k.order_id = ?1
         ORDER BY k.id ASC"
    ).map_err(|e| e.to_string())?;
    
    let kot_iter = stmt.query_map([order_id], |row| {
        Ok((
            row.get::<_, i64>(0)?,
            row.get::<_, Option<String>>(1)?,
            row.get::<_, String>(2)?,
            row.get::<_, i32>(3)?,
            row.get::<_, String>(4)?,
        ))
    }).map_err(|e| e.to_string())?;
    
    let mut kots = Vec::new();
    for kt in kot_iter {
        let (kot_id, table_name, status, print_count, created_at) = kt.map_err(|e| e.to_string())?;
        
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
            print_count,
            created_at,
            items,
        });
    }
    
    Ok(kots)
}

#[tauri::command]
fn increment_kot_print_count(
    kot_id: i64,
    username: String,
    state: tauri::State<'_, db::DbPathState>,
) -> Result<i32, String> {
    let mut conn = rusqlite::Connection::open(&state.path).map_err(|e| e.to_string())?;
    let tx = conn.transaction().map_err(|e| e.to_string())?;
    
    let current_count: i32 = tx.query_row(
        "SELECT print_count FROM kot WHERE id = ?1",
        params![kot_id],
        |row| row.get(0),
    ).map_err(|e| e.to_string())?;
    
    let new_count = current_count + 1;
    tx.execute(
        "UPDATE kot SET print_count = ?1 WHERE id = ?2",
        params![new_count, kot_id],
    ).map_err(|e| e.to_string())?;
    
    if current_count > 0 {
        log_audit(&tx, &username, "kot_reprint", "kot", Some(kot_id), Some(&format!("Reprinted KOT. Previous print count: {}", current_count)))?;
    } else {
        log_audit(&tx, &username, "kot_print", "kot", Some(kot_id), Some("First print of KOT"))?;
    }
    
    tx.commit().map_err(|e| e.to_string())?;
    Ok(new_count)
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
        "SELECT o.id, o.table_id, t.name, o.customer_id, c.name, o.subtotal, o.tax, o.discount, o.service_charge, o.round_off, o.total, o.status, o.payment_mode, o.notes, o.hold_name, o.created_at, o.cancelled_by, o.cancelled_at, o.cancel_reason
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
            cancelled_by: row.get(16)?,
            cancelled_at: row.get(17)?,
            cancel_reason: row.get(18)?,
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
        "SELECT o.id, o.table_id, t.name, o.customer_id, c.name, o.subtotal, o.tax, o.discount, o.service_charge, o.round_off, o.total, o.status, o.payment_mode, o.notes, o.hold_name, o.created_at, o.cancelled_by, o.cancelled_at, o.cancel_reason
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
            cancelled_by: row.get(16)?,
            cancelled_at: row.get(17)?,
            cancel_reason: row.get(18)?,
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
            verify_credentials,
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
            cancel_order_item,
            get_tables,
            transfer_table,
            merge_tables,
            get_active_kots,
            update_kot_status,
            get_kot_by_id,
            get_kots_for_order,
            increment_kot_print_count,
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

#[cfg(test)]
mod tests {
    use super::*;
    use db::setup_test_db;

    #[test]
    fn test_kot_transitions() {
        let cases = vec![
            ("Pending", "Preparing", true),
            ("Preparing", "Ready", true),
            ("Ready", "Completed", true),
            ("Pending", "Ready", false),
            ("Preparing", "Completed", false),
            ("Completed", "Pending", false),
            ("Ready", "Preparing", false),
        ];

        for (from, to, expected) in cases {
            let is_valid = match (from, to) {
                ("Pending", "Preparing") => true,
                ("Preparing", "Ready") => true,
                ("Ready", "Completed") => true,
                _ => false,
            };
            assert_eq!(
                is_valid, expected,
                "Transition from {} to {} should be {}",
                from, to, expected
            );
        }
    }

    #[test]
    fn test_database_migrations_and_cancellations_schema() {
        let conn = setup_test_db();

        // 1. Verify schema tables exist
        let tables: Vec<String> = conn
            .prepare("SELECT name FROM sqlite_master WHERE type='table'")
            .unwrap()
            .query_map([], |row| row.get(0))
            .unwrap()
            .map(|r| r.unwrap())
            .collect();

        assert!(tables.contains(&"order_item_cancellations".to_string()));
        assert!(tables.contains(&"audit_logs".to_string()));

        // 2. Verify cancellation columns in orders table
        let mut stmt = conn.prepare("PRAGMA table_info(orders)").unwrap();
        let cols: Vec<String> = stmt
            .query_map([], |row| row.get(1))
            .unwrap()
            .map(|r| r.unwrap())
            .collect();

        assert!(cols.contains(&"cancelled_by".to_string()));
        assert!(cols.contains(&"cancelled_at".to_string()));
        assert!(cols.contains(&"cancel_reason".to_string()));

        // 3. Verify kot_id column in order_items table
        let mut stmt = conn.prepare("PRAGMA table_info(order_items)").unwrap();
        let cols_items: Vec<String> = stmt
            .query_map([], |row| row.get(1))
            .unwrap()
            .map(|r| r.unwrap())
            .collect();

        assert!(cols_items.contains(&"kot_id".to_string()));
    }

    #[test]
    fn test_user_authentication_roles() {
        let conn = setup_test_db();

        // Check seeded users
        struct User {
            _username: String,
            role: String,
        }

        let mut stmt = conn.prepare("SELECT username, role FROM users").unwrap();
        let users: Vec<User> = stmt
            .query_map([], |row| {
                Ok(User {
                    _username: row.get(0)?,
                    role: row.get(1)?,
                })
            })
            .unwrap()
            .map(|r| r.unwrap())
            .collect();

        // We expect Owner, Manager, Cashier roles
        assert!(users.iter().any(|u| u.role == "Owner"));
        assert!(users.iter().any(|u| u.role == "Manager"));
        assert!(users.iter().any(|u| u.role == "Cashier"));
    }
}
