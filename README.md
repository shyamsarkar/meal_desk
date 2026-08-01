# MealDesk POS

MealDesk is a premium, offline-first desktop Point-of-Sale (POS) and restaurant billing application built with **Rust**, **Tauri v2**, and **SQLite**.

---

## 🚀 Getting Started

### 1. Prerequisites

Ensure you have the following installed on your system:
* **Node.js** (v18 or higher)
* **Rust** (v1.75 or higher)

#### Linux (Debian/Ubuntu/Mint) System Dependencies
Before compiling, you need to install GTK, WebKit2GTK, and other development dependencies:
```bash
sudo apt-get update && sudo apt-get install -y libwebkit2gtk-4.1-dev build-essential curl wget file libssl-dev libgtk-3-dev libayatana-appindicator3-dev librsvg2-dev
```

---

### 2. Development Run

To run the application locally in development mode:

1. Install frontend dependencies:
   ```bash
   npm install
   ```
2. Launch the Tauri development app:
   ```bash
   npm run tauri dev
   ```

---

## 🔑 Default Login Account

The database is pre-seeded with a single default administrator account:

| Username | Password | Access |
| :--- | :--- | :--- |
| **`admin`** | `admin123` | Full application access |

---

## 🗄️ Database Location

MealDesk stores its local data inside a SQLite database file called `mealdesk.db`. 
* On Linux, this is located at: `~/.local/share/com.mealdesk.app/mealdesk.db`
* On Windows, it is located at: `%APPDATA%\com.mealdesk.app\mealdesk.db`

Migrations and default seeds (sample menu items, default configurations) are handled automatically by the Rust core on application startup.

---

## 📦 Packaging (Windows Installer)

Tauri natively builds standard MSI and NSIS installers. To compile the production executable and generate the Windows installer, run the following command **on a Windows machine**:

```bash
npm run tauri build
```

The output installers will be saved in `src-tauri/target/release/bundle/`.

bugs:
- for NC button finalize bill does not work
- for no table if I change the table the order is missing
- can we stop printing once we press the button 'finalize bill'