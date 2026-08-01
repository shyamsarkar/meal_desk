mod db;

use serde::{Deserialize, Serialize};
use tauri::Manager;
use tauri_plugin_dialog::DialogExt;
use rusqlite::params;
use rusqlite::OptionalExtension;

// Data Structures
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct UserInfo {
    pub username: String,
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
    pub is_veg: bool,
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

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct PaymentInput {
    pub payment_method: String,
    pub amount: f64,
}

// Commands
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
fn login(
    username: String,
    password: String,
    state: tauri::State<'_, db::DbPathState>,
) -> Result<UserInfo, String> {
    let conn = rusqlite::Connection::open(&state.path).map_err(|e| e.to_string())?;
    let username_trimmed = username.trim().to_string();
    let hashed = db::hash_password(&password);

    let mut stmt = conn
        .prepare("SELECT username FROM users WHERE LOWER(username) = LOWER(?1) AND password_hash = ?2")
        .map_err(|e| e.to_string())?;

    let result = stmt.query_row(
        rusqlite::params![username_trimmed, hashed],
        |row| {
            Ok(UserInfo {
                username: row.get(0)?,
            })
        },
    );

    match result {
        Ok(user_info) => {
            // Log successful login to audit_logs
            let now = get_db_timestamp(&conn);
            let _ = conn.execute(
                "INSERT INTO audit_logs (username, action, target_type, target_id, details, created_at)
                 VALUES (?1, 'login_success', 'users', NULL, 'Login successful', ?2)",
                rusqlite::params![user_info.username, now],
            );
            Ok(user_info)
        }
        Err(e) => {
            // Log failed attempt to audit_logs — always works even for unknown usernames
            let now = get_db_timestamp(&conn);
            let details = format!("Failed login attempt for username: '{}'. Error: {}", username_trimmed, e);
            let _ = conn.execute(
                "INSERT INTO audit_logs (username, action, target_type, target_id, details, created_at)
                 VALUES (?1, 'login_failed', 'users', NULL, ?2, ?3)",
                rusqlite::params![username_trimmed, details, now],
            );
            Err("Invalid username or password.".to_string())
        }
    }
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
        .prepare("SELECT id, category_id, name, price, gst_rate, image_path, is_available, is_veg FROM products WHERE category_id = ?1")
        .map_err(|e| e.to_string())?;
    
    let prod_iter = stmt
        .query_map([category_id], |row| {
            let is_available_val: i32 = row.get(6)?;
            let is_veg_val: i32 = row.get(7)?;
            Ok(Product {
                id: row.get(0)?,
                category_id: row.get(1)?,
                name: row.get(2)?,
                price: row.get(3)?,
                gst_rate: row.get(4)?,
                image_path: row.get(5)?,
                is_available: is_available_val != 0,
                is_veg: is_veg_val != 0,
            })
        })
        .map_err(|e| e.to_string())?;
        
    let mut products = Vec::new();
    for prod in prod_iter {
        products.push(prod.map_err(|e| e.to_string())?);
    }
    Ok(products)
}

// Advanced Billing
#[tauri::command]
fn create_order(
    table_id: Option<i64>,
    customer_id: Option<i64>,
    notes: Option<String>,
    items: Vec<OrderItemInput>,
    created_at: String,
    username: String,
    state: tauri::State<'_, db::DbPathState>,
) -> Result<i64, String> {
    let mut conn = rusqlite::Connection::open(&state.path).map_err(|e| e.to_string())?;
    let tx = conn.transaction().map_err(|e| e.to_string())?;
    
    let order_id = {
        tx.execute(
            "INSERT INTO orders (table_id, customer_id, status, notes, created_at)
             VALUES (?1, ?2, 'Pending', ?3, ?4)",
            params![table_id, customer_id, notes, created_at],
        ).map_err(|e| e.to_string())?;
        tx.last_insert_rowid()
    };
    
    log_audit(&tx, &username, "create_order", "orders", Some(order_id), Some("Created order in status Pending"))?;
    
    let mut inserted_items = Vec::new();
    for item in &items {
        tx.execute(
            "INSERT INTO order_items (order_id, product_id, name, quantity, price, gst_rate, notes, kot_id)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, NULL)",
            params![order_id, item.product_id, item.name, item.quantity, item.price, item.gst_rate, item.notes],
        ).map_err(|e| e.to_string())?;
        let order_item_id = tx.last_insert_rowid();
        inserted_items.push((order_item_id, item.product_id, item.quantity, item.notes.clone()));
    }
    
    if let Some(tid) = table_id {
        tx.execute(
            "UPDATE tables SET status = 'Occupied' WHERE id = ?1",
            params![tid],
        ).map_err(|e| e.to_string())?;
    }
    
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
    
    tx.commit().map_err(|e| e.to_string())?;
    Ok(order_id)
}

#[tauri::command]
fn update_order(
    order_id: i64,
    table_id: Option<i64>,
    customer_id: Option<i64>,
    notes: Option<String>,
    items: Vec<OrderItemInput>,
    created_at: String,
    username: String,
    state: tauri::State<'_, db::DbPathState>,
) -> Result<(), String> {
    let mut conn = rusqlite::Connection::open(&state.path).map_err(|e| e.to_string())?;
    let tx = conn.transaction().map_err(|e| e.to_string())?;

    // 1. Fetch old table_id and old_status
    let (old_table_id, old_status): (Option<i64>, String) = tx.query_row(
        "SELECT table_id, status FROM orders WHERE id = ?1",
        [order_id],
        |row| Ok((row.get(0)?, row.get(1)?)),
    ).map_err(|e| e.to_string())?;

    if old_status == "Billed" || old_status == "Completed" || old_status == "Cancelled" {
        return Err("Cannot update order in its current final status".to_string());
    }

    log_audit(&tx, &username, "update_order", "orders", Some(order_id), Some(&format!("Updating order. Old status: {}", old_status)))?;

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
                // Not sent to KOT yet
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
        }
    }

    // 4. Handle deleted items
    for (&db_item_id, db_item) in &db_items {
        if !processed_incoming_ids.contains(&db_item_id) {
            if db_item.kot_id.is_some() {
                return Err(format!("Cannot delete order item ID {} because it was already sent to the kitchen.", db_item_id));
            }
            tx.execute("DELETE FROM order_items WHERE id = ?1", [db_item_id]).map_err(|e| e.to_string())?;
            log_audit(&tx, &username, "delete_item", "order_items", Some(db_item_id), Some(&format!("Deleted unsent item from order {}", order_id)))?;
        }
    }

    // 5. Update orders table row
    tx.execute(
        "UPDATE orders SET table_id = ?1, customer_id = ?2, notes = ?3 WHERE id = ?4",
        params![table_id, customer_id, notes, order_id],
    ).map_err(|e| e.to_string())?;

    // 6. KOT Generation for Unsent Items
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

    // 7. Table Status Management
    if let Some(old_tid) = old_table_id {
        if table_id != Some(old_tid) {
            tx.execute(
                "UPDATE tables SET status = 'Free', merged_into = NULL WHERE id = ?1 OR merged_into = ?1",
                [old_tid],
            ).map_err(|e| e.to_string())?;
        }
    }
    
    if let Some(tid) = table_id {
        tx.execute(
            "UPDATE tables SET status = 'Occupied' WHERE id = ?1",
            [tid],
        ).map_err(|e| e.to_string())?;
    }

    tx.commit().map_err(|e| e.to_string())?;
    Ok(())
}

const ORDERS_SELECT_SQL: &str = 
    "SELECT o.id, o.table_id, t.name, o.customer_id, c.name, 
            COALESCE(b.subtotal, 0.0), 
            COALESCE(b.tax, 0.0), 
            COALESCE(b.discount, 0.0), 
            COALESCE(b.service_charge, 0.0), 
            COALESCE(b.round_off, 0.0), 
            COALESCE(b.total, 0.0), 
            o.status, 
            (SELECT GROUP_CONCAT(payment_method, ', ') FROM payments WHERE bill_id = b.id), 
            o.notes, o.created_at, o.cancelled_by, o.cancelled_at, o.cancel_reason
     FROM orders o 
     LEFT JOIN tables t ON o.table_id = t.id 
     LEFT JOIN customers c ON o.customer_id = c.id
     LEFT JOIN bills b ON o.id = b.order_id";

fn fetch_orders_by_query(
    conn: &rusqlite::Connection,
    sql_query: &str,
    params: &[&dyn rusqlite::ToSql],
) -> Result<Vec<OrderHeader>, String> {
    let mut stmt = conn.prepare(sql_query).map_err(|e| e.to_string())?;
    
    let order_iter = stmt.query_map(params, |row| {
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
            created_at: row.get(14)?,
            cancelled_by: row.get(15)?,
            cancelled_at: row.get(16)?,
            cancel_reason: row.get(17)?,
        })
    }).map_err(|e| e.to_string())?;
    
    let mut orders = Vec::new();
    for ord in order_iter {
        let mut header = ord.map_err(|e| e.to_string())?;
        
        // If the order is pending, dynamically compute its current items' totals
        if header.status == "Pending" {
            let mut item_stmt = conn.prepare(
                "SELECT quantity, 
                        COALESCE((SELECT SUM(c.quantity) FROM order_item_cancellations c WHERE c.order_item_id = oi.id), 0) AS cancelled_qty,
                        price, gst_rate
                 FROM order_items oi
                 WHERE order_id = ?1"
            ).map_err(|e| e.to_string())?;
            
            struct ItemCalc {
                qty: i64,
                cancelled: i64,
                price: f64,
                gst: f64,
            }
            
            let item_iter = item_stmt.query_map([header.id], |row| {
                Ok(ItemCalc {
                    qty: row.get(0)?,
                    cancelled: row.get(1)?,
                    price: row.get(2)?,
                    gst: row.get(3)?,
                })
            }).map_err(|e| e.to_string())?;
            
            let mut subtotal = 0.0;
            let mut total_tax = 0.0;
            for it in item_iter {
                let item = it.map_err(|e| e.to_string())?;
                let effective_qty = item.qty - item.cancelled;
                if effective_qty > 0 {
                    let item_subtotal = item.price * (effective_qty as f64);
                    let item_tax = item_subtotal * (item.gst / 100.0);
                    subtotal += item_subtotal;
                    total_tax += item_tax;
                }
            }
            let total_raw = subtotal + total_tax;
            let total_rounded = total_raw.round();
            let round_off = total_rounded - total_raw;
            
            header.subtotal = subtotal;
            header.tax = total_tax;
            header.round_off = round_off;
            header.total = total_rounded;
        }
        
        orders.push(header);
    }
    
    Ok(orders)
}

#[tauri::command]
fn get_order(order_id: i64, state: tauri::State<'_, db::DbPathState>) -> Result<OrderOutput, String> {
    let conn = rusqlite::Connection::open(&state.path).map_err(|e| e.to_string())?;
    
    let mut orders = fetch_orders_by_query(&conn, &format!("{} WHERE o.id = ?1", ORDERS_SELECT_SQL), &[&order_id])?;
    if orders.is_empty() {
        return Err("Order not found".to_string());
    }
    let header = orders.remove(0);
    
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
fn get_completed_orders(state: tauri::State<'_, db::DbPathState>) -> Result<Vec<OrderHeader>, String> {
    let conn = rusqlite::Connection::open(&state.path).map_err(|e| e.to_string())?;
    fetch_orders_by_query(
        &conn,
        &format!("{} WHERE o.status = 'Completed' ORDER BY o.id DESC", ORDERS_SELECT_SQL),
        &[]
    )
}

#[tauri::command]
fn get_customer_orders(customer_id: i64, state: tauri::State<'_, db::DbPathState>) -> Result<Vec<OrderHeader>, String> {
    let conn = rusqlite::Connection::open(&state.path).map_err(|e| e.to_string())?;
    fetch_orders_by_query(
        &conn,
        &format!("{} WHERE o.customer_id = ?1 ORDER BY o.id DESC", ORDERS_SELECT_SQL),
        &[&customer_id]
    )
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
    
    let mut kot_cancellations = Vec::new();
    {
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
                // index matches: product_id (0), name (1), quantity (2), cancelled_qty (3), notes (4), kot_id (5)
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
            if effective_qty > 0 && item.kot_id.is_some() {
                kot_cancellations.push((item.product_id, effective_qty, item.notes));
            }
        }
    }
    
    // Set status to Cancelled and record cancellation info
    let now = get_db_timestamp(&tx);
    tx.execute(
        "UPDATE orders SET status = 'Cancelled', cancelled_by = ?1, cancelled_at = ?2, cancel_reason = ?3 WHERE id = ?4",
        params![cancelled_by, now, reason, order_id],
    ).map_err(|e| e.to_string())?;
    
    // Set bill status to Cancelled if a bill exists
    tx.execute(
        "UPDATE bills SET status = 'Cancelled' WHERE order_id = ?1",
        [order_id],
    ).map_err(|e| e.to_string())?;
    
    // Free table and reset merged_into
    if let Some(tid) = table_id {
        tx.execute(
            "UPDATE tables SET status = 'Free', merged_into = NULL WHERE id = ?1 OR merged_into = ?1",
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
    
    // If a bill exists, update its totals
    let bill_exists: bool = tx.query_row(
        "SELECT COUNT(*) FROM bills WHERE order_id = ?1",
        [item.order_id],
        |row| row.get::<_, i64>(0),
    ).unwrap_or(0) > 0;
    
    if bill_exists {
        let (discount, service_charge): (f64, f64) = tx.query_row(
            "SELECT discount, service_charge FROM bills WHERE order_id = ?1",
            [item.order_id],
            |row| Ok((row.get(0)?, row.get(1)?)),
        ).map_err(|e| e.to_string())?;
        
        let (subtotal, final_tax, round_off, total_rounded) = calculate_totals_for_order(&tx, item.order_id, discount, service_charge)?;
        
        tx.execute(
            "UPDATE bills 
             SET subtotal = ?1, tax = ?2, round_off = ?3, total = ?4 
             WHERE order_id = ?5",
            params![subtotal, final_tax, round_off, total_rounded, item.order_id],
        ).map_err(|e| e.to_string())?;
    }
    
    tx.commit().map_err(|e| e.to_string())?;
    Ok(())
}

fn calculate_totals_for_order(
    conn: &rusqlite::Connection,
    order_id: i64,
    discount: f64,
    service_charge: f64,
) -> Result<(f64, f64, f64, f64), String> {
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
    
    Ok((subtotal, final_tax, round_off, total_rounded))
}

#[tauri::command]
fn generate_bill(
    order_id: i64,
    discount: f64,
    service_charge: f64,
    username: String,
    state: tauri::State<'_, db::DbPathState>,
) -> Result<OrderHeader, String> {
    let mut conn = rusqlite::Connection::open(&state.path).map_err(|e| e.to_string())?;
    let tx = conn.transaction().map_err(|e| e.to_string())?;
    
    // Check if bill already exists
    let existing_bill_id: Option<i64> = tx.query_row(
        "SELECT id FROM bills WHERE order_id = ?1",
        [order_id],
        |row| row.get(0),
    ).optional().map_err(|e| e.to_string())?;
    
    if existing_bill_id.is_none() {
        // Calculate totals
        let (subtotal, final_tax, round_off, total_rounded) = calculate_totals_for_order(&tx, order_id, discount, service_charge)?;
        
        let now = get_db_timestamp(&tx);
        
        // Insert bill record
        tx.execute(
            "INSERT INTO bills (order_id, bill_number, subtotal, discount, tax, service_charge, round_off, total, status, created_at, billed_at)
             VALUES (?1, '', ?2, ?3, ?4, ?5, ?6, ?7, 'Unpaid', ?8, ?9)",
            params![order_id, subtotal, discount, final_tax, service_charge, round_off, total_rounded, now, now],
        ).map_err(|e| e.to_string())?;
        
        let new_bill_id = tx.last_insert_rowid();
        let bill_number = format!("BILL-{:05}", new_bill_id);
        
        tx.execute(
            "UPDATE bills SET bill_number = ?1 WHERE id = ?2",
            params![bill_number, new_bill_id],
        ).map_err(|e| e.to_string())?;
        
        // Update order status to 'Billed'
        tx.execute(
            "UPDATE orders SET status = 'Billed' WHERE id = ?1",
            [order_id],
        ).map_err(|e| e.to_string())?;
        
        // Update table status to 'Billed' if order has table_id
        let table_id: Option<i64> = tx.query_row(
            "SELECT table_id FROM orders WHERE id = ?1",
            [order_id],
            |row| row.get(0),
        ).map_err(|e| e.to_string())?;
        
        if let Some(tid) = table_id {
            tx.execute(
                "UPDATE tables SET status = 'Billed' WHERE id = ?1",
                [tid],
            ).map_err(|e| e.to_string())?;
        }
        
        log_audit(&tx, &username, "generate_bill", "bills", Some(new_bill_id), Some(&format!("Generated bill {} for order {}", bill_number, order_id)))?;
    }
    
    tx.commit().map_err(|e| e.to_string())?;
    
    // Retrieve and return the updated order header
    let conn_read = rusqlite::Connection::open(&state.path).map_err(|e| e.to_string())?;
    let mut orders = fetch_orders_by_query(&conn_read, &format!("{} WHERE o.id = ?1", ORDERS_SELECT_SQL), &[&order_id])?;
    if orders.is_empty() {
        return Err("Order not found".to_string());
    }
    Ok(orders.remove(0))
}

#[tauri::command]
fn record_payments(
    order_id: i64,
    payments: Vec<PaymentInput>,
    username: String,
    state: tauri::State<'_, db::DbPathState>,
) -> Result<(), String> {
    let mut conn = rusqlite::Connection::open(&state.path).map_err(|e| e.to_string())?;
    let tx = conn.transaction().map_err(|e| e.to_string())?;
    
    // Fetch bill id and total using order_id
    let (bill_id, bill_total): (i64, f64) = tx.query_row(
        "SELECT id, total FROM bills WHERE order_id = ?1",
        [order_id],
        |row| Ok((row.get(0)?, row.get(1)?)),
    ).map_err(|e| format!("No bill found for order {}: {}", order_id, e))?;
    
    // Calculate total payments amount
    let mut total_paid = 0.0;
    let mut cash_payment_index: Option<usize> = None;
    
    for (i, payment) in payments.iter().enumerate() {
        if payment.payment_method == "Cash" {
            cash_payment_index = Some(i);
        }
        total_paid += payment.amount;
    }
    
    if total_paid < bill_total {
        return Err(format!("Insufficient payment: received ₹{:.2}, bill total is ₹{:.2}", total_paid, bill_total));
    }
    
    let now = get_db_timestamp(&tx);
    
    // Check if cash change handles overpayment
    if total_paid > bill_total {
        if let Some(cash_idx) = cash_payment_index {
            let change = total_paid - bill_total;
            
            for (i, payment) in payments.iter().enumerate() {
                let mut amt = payment.amount;
                if i == cash_idx {
                    amt -= change;
                }
                
                if amt > 0.0 {
                    tx.execute(
                        "INSERT INTO payments (bill_id, payment_method, amount, created_at)
                         VALUES (?1, ?2, ?3, ?4)",
                        params![bill_id, payment.payment_method, amt, now],
                    ).map_err(|e| e.to_string())?;
                }
            }
        } else {
            return Err("Overpayment is only supported for Cash payments (change return)".to_string());
        }
    } else {
        // Exact payment
        for payment in &payments {
            if payment.amount > 0.0 {
                tx.execute(
                    "INSERT INTO payments (bill_id, payment_method, amount, created_at)
                     VALUES (?1, ?2, ?3, ?4)",
                    params![bill_id, payment.payment_method, payment.amount, now],
                ).map_err(|e| e.to_string())?;
            }
        }
    }
    
    // Update bill status to 'Paid'
    tx.execute(
        "UPDATE bills SET status = 'Paid' WHERE id = ?1",
        [bill_id],
    ).map_err(|e| e.to_string())?;
    
    // Update order status to 'Completed'
    tx.execute(
        "UPDATE orders SET status = 'Completed' WHERE id = ?1",
        [order_id],
    ).map_err(|e| e.to_string())?;
    
    // Fetch table_id and customer_id
    let (table_id, customer_id): (Option<i64>, Option<i64>) = tx.query_row(
        "SELECT table_id, customer_id FROM orders WHERE id = ?1",
        [order_id],
        |row| Ok((row.get(0)?, row.get(1)?)),
    ).map_err(|e| e.to_string())?;
    
    // Free table and reset merged_into
    if let Some(tid) = table_id {
        tx.execute(
            "UPDATE tables SET status = 'Free', merged_into = NULL WHERE id = ?1 OR merged_into = ?1",
            [tid],
        ).map_err(|e| e.to_string())?;
    }
    
    // Award loyalty points
    if let Some(cid) = customer_id {
        let pts_gained = (bill_total / 100.0) as i64;
        tx.execute(
            "UPDATE customers SET loyalty_points = loyalty_points + ?1 WHERE id = ?2",
            params![pts_gained, cid],
        ).map_err(|e| e.to_string())?;
    }
    
    log_audit(&tx, &username, "record_payments", "bills", Some(bill_id), Some(&format!("Recorded payment of ₹{:.2} for bill ID {}", bill_total, bill_id)))?;
    
    tx.commit().map_err(|e| e.to_string())?;
    Ok(())
}

// Table Management
#[tauri::command]
fn get_tables(state: tauri::State<'_, db::DbPathState>) -> Result<Vec<TableDetails>, String> {
    let conn = rusqlite::Connection::open(&state.path).map_err(|e| e.to_string())?;
    let mut stmt = conn.prepare(
        "SELECT t.id, t.name, t.status, t.merged_into,
                o.id AS current_order_id,
                COALESCE(
                    b.total,
                    (SELECT COALESCE(SUM(oi.price * (oi.quantity - COALESCE((SELECT SUM(c.quantity) FROM order_item_cancellations c WHERE c.order_item_id = oi.id), 0))), 0.0)
                     FROM order_items oi WHERE oi.order_id = o.id)
                ) as current_order_total
         FROM tables t
         LEFT JOIN orders o ON t.id = o.table_id AND o.status IN ('Pending', 'Billed')
         LEFT JOIN bills b ON o.id = b.order_id
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
        "SELECT id FROM orders WHERE table_id = ?1 AND status IN ('Pending', 'Billed')",
        [from_table_id],
        |row| row.get(0),
    ).optional().map_err(|e| e.to_string())?;
    
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
    
    // Fetch source status to apply to target table
    let source_status: String = tx.query_row(
        "SELECT status FROM tables WHERE id = ?1",
        [from_table_id],
        |row| row.get(0),
    ).map_err(|e| e.to_string())?;
    
    // Reset source table
    tx.execute(
        "UPDATE tables SET status = 'Free', merged_into = NULL WHERE id = ?1 OR merged_into = ?1",
        [from_table_id],
    ).map_err(|e| e.to_string())?;
    
    // Set target table status
    tx.execute(
        "UPDATE tables SET status = ?1 WHERE id = ?2",
        params![source_status, to_table_id],
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
        "UPDATE tables SET merged_into = ?1, status = 'Free' WHERE id = ?2",
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

#[tauri::command]
fn delete_category(
    id: i64,
    state: tauri::State<'_, db::DbPathState>,
) -> Result<(), String> {
    let conn = rusqlite::Connection::open(&state.path).map_err(|e| e.to_string())?;

    // Check if any products exist under this category
    let product_count: i64 = conn.query_row(
        "SELECT COUNT(*) FROM products WHERE category_id = ?1",
        [id],
        |row| row.get(0),
    ).unwrap_or(0);

    if product_count > 0 {
        return Err(format!(
            "Cannot delete category: {} product(s) still belong to it. Delete or reassign products first.",
            product_count
        ));
    }

    conn.execute("DELETE FROM categories WHERE id = ?1", [id])
        .map_err(|e| e.to_string())?;
    Ok(())
}

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
fn delete_product(
    id: i64,
    state: tauri::State<'_, db::DbPathState>,
) -> Result<(), String> {
    let conn = rusqlite::Connection::open(&state.path).map_err(|e| e.to_string())?;

    // Check if product has been used in any order items
    let order_count: i64 = conn.query_row(
        "SELECT COUNT(*) FROM order_items WHERE product_id = ?1",
        [id],
        |row| row.get(0),
    ).unwrap_or(0);

    if order_count > 0 {
        return Err(format!(
            "Cannot delete product: it has been used in {} order(s). Mark it as unavailable instead.",
            order_count
        ));
    }

    conn.execute("DELETE FROM products WHERE id = ?1", [id])
        .map_err(|e| e.to_string())?;
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
        conn.execute(
            "INSERT INTO products (category_id, name, price, gst_rate, is_available) VALUES (?1, ?2, ?3, ?4, ?5)",
            params![category_id, name, price, gst_rate, avail_val],
        ).map_err(|e| e.to_string())?;
    }
    Ok(())
}

// Customers Profile Management
#[tauri::command]
fn get_customers(state: tauri::State<'_, db::DbPathState>) -> Result<Vec<CustomerDetails>, String> {
    let conn = rusqlite::Connection::open(&state.path).map_err(|e| e.to_string())?;
    let mut stmt = conn
        .prepare("SELECT id, name, phone, email, loyalty_points FROM customers ORDER BY name ASC")
        .map_err(|e| e.to_string())?;
    
    let cust_iter = stmt
        .query_map([], |row| {
            Ok(CustomerDetails {
                id: row.get(0)?,
                name: row.get(1)?,
                phone: row.get(2)?,
                email: row.get(3)?,
                loyalty_points: row.get(4)?,
            })
        })
        .map_err(|e| e.to_string())?;
        
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
) -> Result<(), String> {
    let conn = rusqlite::Connection::open(&state.path).map_err(|e| e.to_string())?;

    if let Some(cust_id) = id {
        if let Some(pts) = loyalty_points {
            // Full update including loyalty points when explicitly provided
            conn.execute(
                "UPDATE customers SET name = ?1, phone = ?2, email = ?3, loyalty_points = ?4 WHERE id = ?5",
                params![name, phone, email, pts, cust_id],
            ).map_err(|e| e.to_string())?;
        } else {
            // Update without touching loyalty points
            conn.execute(
                "UPDATE customers SET name = ?1, phone = ?2, email = ?3 WHERE id = ?4",
                params![name, phone, email, cust_id],
            ).map_err(|e| e.to_string())?;
        }
    } else {
        let pts = loyalty_points.unwrap_or(0);
        conn.execute(
            "INSERT INTO customers (name, phone, email, loyalty_points) VALUES (?1, ?2, ?3, ?4)",
            params![name, phone, email, pts],
        ).map_err(|e| e.to_string())?;
    }
    Ok(())
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
        "SELECT COALESCE(SUM(b.total), 0.0), COALESCE(SUM(b.tax), 0.0), COUNT(o.id)
         FROM orders o
         JOIN bills b ON o.id = b.order_id
         WHERE o.status = 'Completed' AND date(o.created_at) >= date(?1) AND date(o.created_at) <= date(?2)"
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
fn backup_db(app: tauri::AppHandle, state: tauri::State<'_, db::DbPathState>) -> Result<(), String> {
    let db_path = state.path.clone();

    // Build a default filename with a timestamp: mealdesk_backup_YYYYMMDD_HHMMSS.db
    let now = chrono::Local::now();
    let default_name = now.format("mealdesk_backup_%Y%m%d_%H%M%S.db").to_string();

    let save_path = app
        .dialog()
        .file()
        .set_title("Save Database Backup")
        .set_file_name(&default_name)
        .add_filter("SQLite Database", &["db"])
        .blocking_save_file();

    match save_path {
        Some(path) => {
            let dest = path.to_string();
            std::fs::copy(&db_path, &dest)
                .map(|_| ())
                .map_err(|e| format!("Backup failed: {}", e))
        }
        None => Err("Backup cancelled".to_string()),
    }
}

#[tauri::command]
fn restore_db(source_path: String, state: tauri::State<'_, db::DbPathState>) -> Result<(), String> {
    std::fs::copy(source_path, &state.path).map(|_| ()).map_err(|e| e.to_string())
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
    
    let get_sales_by_method = |method: &str| -> f64 {
        conn.query_row(
            "SELECT COALESCE(SUM(p.amount), 0.0) 
             FROM payments p
             JOIN bills b ON p.bill_id = b.id
             JOIN orders o ON b.order_id = o.id
             WHERE o.status = 'Completed' AND p.payment_method = ?1 AND date(o.created_at) >= date(?2) AND date(o.created_at) <= date(?3)",
            [method, &start_date, &end_date],
            |row| row.get(0),
        ).unwrap_or(0.0)
    };
    
    let cash = get_sales_by_method("Cash");
    let upi = get_sales_by_method("UPI");
    let card = get_sales_by_method("Card");
    
    let mixed: f64 = conn.query_row(
        "SELECT COALESCE(SUM(b.total), 0.0)
         FROM bills b
         JOIN orders o ON b.order_id = o.id
         WHERE o.status = 'Completed' 
           AND (SELECT COUNT(DISTINCT payment_method) FROM payments WHERE bill_id = b.id) > 1
           AND date(o.created_at) >= date(?1) AND date(o.created_at) <= date(?2)",
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
        .plugin(tauri_plugin_dialog::init())
        .setup(|app| {
            let db_path = db::init_db(app.handle())?;
            app.manage(db::DbPathState { path: db_path });
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            login,
            get_restaurant_info,
            update_restaurant_info,
            get_categories,
            get_products_by_category,
            create_order,
            update_order,
            get_order,
            cancel_order,
            cancel_order_item,
            generate_bill,
            record_payments,
            get_tables,
            transfer_table,
            merge_tables,
            get_active_kots,
            update_kot_status,
            get_kot_by_id,
            get_kots_for_order,
            increment_kot_print_count,
            upsert_category,
            delete_category,
            upsert_product,
            delete_product,
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
        assert!(tables.contains(&"bills".to_string()));
        assert!(tables.contains(&"payments".to_string()));
        assert!(!tables.contains(&"inventory".to_string()));

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
        assert!(!cols.contains(&"total".to_string()));
        assert!(!cols.contains(&"hold_name".to_string()));

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
    fn test_admin_authentication_and_no_roles() {
        let conn = setup_test_db();

        // Check seeded users
        struct User {
            username: String,
        }

        let mut stmt = conn.prepare("SELECT username FROM users").unwrap();
        let users: Vec<User> = stmt
            .query_map([], |row| {
                Ok(User {
                    username: row.get(0)?,
                })
            })
            .unwrap()
            .map(|r| r.unwrap())
            .collect();

        assert_eq!(users.len(), 1);
        assert_eq!(users[0].username, "admin");

        // Verify the admin password is hash of "admin123"
        let hashed_pw: String = conn
            .query_row(
                "SELECT password_hash FROM users WHERE username = 'admin'",
                [],
                |row| row.get(0),
            )
            .unwrap();
        assert_eq!(hashed_pw, db::hash_password("admin123"));

        // Verify the users table schema only contains id, username, and password_hash
        let mut pragma_stmt = conn.prepare("PRAGMA table_info(users)").unwrap();
        let cols: Vec<String> = pragma_stmt
            .query_map([], |row| row.get(1))
            .unwrap()
            .map(|r| r.unwrap())
            .collect();
        assert!(!cols.contains(&"role".to_string()));
        assert_eq!(cols.len(), 3);
        assert!(cols.contains(&"id".to_string()));
        assert!(cols.contains(&"username".to_string()));
        assert!(cols.contains(&"password_hash".to_string()));
    }
}
