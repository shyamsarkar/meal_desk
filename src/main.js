const { invoke } = window.__TAURI__.core;

// Memory State
let currentUser = null;
let restaurantInfo = null;
let categories = [];
let activeCategoryId = null;
let products = [];
let cart = [];
let tables = [];
let customers = [];
let activeOrderId = null; // Stored if resuming an existing order

// DOM Elements
let loginScreen, appScreen, loginForm, usernameInput, passwordInput, loginError;
let displayUserName, navItems, panels, activePanelTitle, appClock, restaurantNameHeader;

// POS Elements
let posCategoriesContainer, posProductsContainer, productSearchInput, cartItemsList;
let cartSubtotalEl, cartTaxEl, cartTotalEl, checkoutBtn, clearCartBtn;
let cartDiscountInput, cartServiceInput, cartCustomerSelect, cartTableSelect;

// Table management elements
let tablesGrid, tableTransferBtn, tableMergeBtn;

// Kitchen elements
let kitchenKotGrid;

// Menu editor elements
let menuEditorCategories, menuEditorProductsList, addCategoryBtn, addProductBtn;

let activeOrderStatus = null;

// Customer database elements
let customersList, addCustomerBtn;

// Reports elements
let reportFromDate, reportToDate, generateReportBtn;
let reportTotalRevenue, reportTotalTax, reportOrderCount, reportAvgBill;
let reportCashSales, reportUpiSales, reportCardSales, reportMixedSales, reportProductsBody, reportCompletedBillsList;

// Settings elements
let settingsRestForm, settingsInputName, settingsInputGstin, settingsInputAddress, settingsInputPhone, settingsInputEmail, settingsInputFooter;
let settingsThemeSelect, settingsGstDefault, settingsPrinterSelect;
let settingsBackupPath, settingsBtnBackup, settingsRestorePath, settingsBtnRestore;

// Modal elements
let successModal, modalTitle, modalMsg, modalCloseBtn;
let itemNotesModal, notesProductName, itemNotesText, notesModalCancel, notesModalSave, activeNotesItemIndex;
let tablesActionModal, tableActionTitle, tableActionFromSelect, tableActionToSelect, tableActionCancel, tableActionSubmit, activeTableActionType;
let categoryModal, categoryModalTitle, categoryModalName, categoryModalDesc, categoryModalCancel, categoryModalSave, activeCategoryEditId;
let productModal, productModalTitle, productModalCategory, productModalName, productModalPrice, productModalGst, productModalAvailable, productModalCancel, productModalSave, activeProductEditId;
let customerModal, customerModalTitle, customerModalName, customerModalPhone, customerModalEmail, customerModalCancel, customerModalSave, activeCustomerEditId;
let customerHistoryModal, custHistoryName, custHistoryList, custHistoryClose;
let checkoutModal, checkoutModalAmount, checkoutPaymentMode, checkoutSplitBlock, checkoutChangeBlock, checkoutChangeRow, checkoutChangeAmount, checkoutCashReceived, checkoutModalCancel, checkoutModalPrint, checkoutModalConfirm;
let splitCashAmount, splitUpiAmount, splitCardAmount, splitRemainingTotal;
let receiptStoreName, receiptStoreAddress, receiptStorePhone, receiptStoreGstin, receiptBillNumber, receiptDate, receiptTable, receiptCashier, receiptCustomerRow, receiptCustomer, receiptItemsBody, receiptSubtotal, receiptDiscount, receiptService, receiptTax, receiptTotal, receiptFooterMsg;

// Cancellation Modal
let cancelModal, cancelModalTitle, cancelModalItemName;
let cancelQtyGroup, cancelQtyInput, cancelReasonSelect, cancelReasonTextGroup, cancelReasonText, cancelModalCancelBtn, cancelModalSubmitBtn;
let cancelOrderBtn;
let activeCancellationTarget = null; // { type: 'item'|'order', itemIndex: number, orderItem: object }

// Entry
window.addEventListener("DOMContentLoaded", () => {
  initDOMElements();
  setupEventListeners();
  loadSavedPreferences();
  startClock();

  // Restore saved session if exists
  const savedUser = localStorage.getItem("mealdesk_user");
  if (savedUser) {
    try {
      const user = JSON.parse(savedUser);
      if (user && user.username) {
        handleLoginSuccess(user);
      }
    } catch (e) {
      localStorage.removeItem("mealdesk_user");
    }
  }
});

function initDOMElements() {
  loginScreen = document.getElementById("login-screen");
  appScreen = document.getElementById("app-screen");
  loginForm = document.getElementById("login-form");
  usernameInput = document.getElementById("username");
  passwordInput = document.getElementById("password");
  loginError = document.getElementById("login-error");
  displayUserName = document.getElementById("display-user-name");
  navItems = document.querySelectorAll(".nav-item");
  panels = document.querySelectorAll(".panel");
  activePanelTitle = document.getElementById("active-panel-title");
  appClock = document.getElementById("app-clock");
  restaurantNameHeader = document.getElementById("restaurant-name-header");

  // POS
  posCategoriesContainer = document.getElementById("pos-categories");
  posProductsContainer = document.getElementById("pos-products");
  productSearchInput = document.getElementById("product-search");
  cartItemsList = document.getElementById("cart-items-list");
  cartSubtotalEl = document.getElementById("cart-subtotal");
  cartTaxEl = document.getElementById("cart-tax");
  cartTotalEl = document.getElementById("cart-total");
  checkoutBtn = document.getElementById("checkout-btn");
  clearCartBtn = document.getElementById("clear-cart-btn");
  cartDiscountInput = document.getElementById("cart-discount-input");
  cartServiceInput = document.getElementById("cart-service-input");
  cartCustomerSelect = document.getElementById("cart-customer-select");
  cartTableSelect = document.getElementById("cart-table-select");

  // Tables
  tablesGrid = document.getElementById("tables-grid");
  tableTransferBtn = document.getElementById("table-transfer-btn");
  tableMergeBtn = document.getElementById("table-merge-btn");

  // Kitchen
  kitchenKotGrid = document.getElementById("kitchen-kot-grid");

  // Menu editor
  menuEditorCategories = document.getElementById("menu-editor-categories");
  menuEditorProductsList = document.getElementById("menu-editor-products-list");
  addCategoryBtn = document.getElementById("add-category-btn");
  addProductBtn = document.getElementById("add-product-btn");



  // Customers
  customersList = document.getElementById("customers-list");
  addCustomerBtn = document.getElementById("add-customer-btn");

  // Reports
  reportFromDate = document.getElementById("report-from-date");
  reportToDate = document.getElementById("report-to-date");
  generateReportBtn = document.getElementById("generate-report-btn");
  reportTotalRevenue = document.getElementById("report-total-revenue");
  reportTotalTax = document.getElementById("report-total-tax");
  reportOrderCount = document.getElementById("report-order-count");
  reportAvgBill = document.getElementById("report-avg-bill");
  reportCashSales = document.getElementById("report-cash-sales");
  reportUpiSales = document.getElementById("report-upi-sales");
  reportCardSales = document.getElementById("report-card-sales");
  reportMixedSales = document.getElementById("report-mixed-sales");
  reportProductsBody = document.getElementById("report-products-body");
  reportCompletedBillsList = document.getElementById("report-completed-bills-list");

  // Settings
  settingsRestForm = document.getElementById("settings-rest-form");
  settingsInputName = document.getElementById("settings-input-name");
  settingsInputGstin = document.getElementById("settings-input-gstin");
  settingsInputAddress = document.getElementById("settings-input-address");
  settingsInputPhone = document.getElementById("settings-input-phone");
  settingsInputEmail = document.getElementById("settings-input-email");
  settingsInputFooter = document.getElementById("settings-input-footer");
  settingsThemeSelect = document.getElementById("settings-theme-select");
  settingsGstDefault = document.getElementById("settings-gst-default");
  settingsPrinterSelect = document.getElementById("settings-printer-select");
  settingsBackupPath = document.getElementById("settings-backup-path");
  settingsBtnBackup = document.getElementById("settings-btn-backup");
  settingsRestorePath = document.getElementById("settings-restore-path");
  settingsBtnRestore = document.getElementById("settings-btn-restore");

  // Modals
  successModal = document.getElementById("success-modal");
  modalTitle = document.getElementById("modal-title");
  modalMsg = document.getElementById("modal-msg");
  modalCloseBtn = document.getElementById("modal-close-btn");



  itemNotesModal = document.getElementById("item-notes-modal");
  notesProductName = document.getElementById("notes-product-name");
  itemNotesText = document.getElementById("item-notes-text");
  notesModalCancel = document.getElementById("notes-modal-cancel");
  notesModalSave = document.getElementById("notes-modal-save");

  tablesActionModal = document.getElementById("tables-action-modal");
  tableActionTitle = document.getElementById("table-action-title");
  tableActionFromSelect = document.getElementById("table-action-from-select");
  tableActionToSelect = document.getElementById("table-action-to-select");
  tableActionCancel = document.getElementById("table-action-cancel");
  tableActionSubmit = document.getElementById("table-action-submit");

  categoryModal = document.getElementById("category-modal");
  categoryModalTitle = document.getElementById("category-modal-title");
  categoryModalName = document.getElementById("category-modal-name");
  categoryModalDesc = document.getElementById("category-modal-desc");
  categoryModalCancel = document.getElementById("category-modal-cancel");
  categoryModalSave = document.getElementById("category-modal-save");

  productModal = document.getElementById("product-modal");
  productModalTitle = document.getElementById("product-modal-title");
  productModalCategory = document.getElementById("product-modal-category");
  productModalName = document.getElementById("product-modal-name");
  productModalPrice = document.getElementById("product-modal-price");
  productModalGst = document.getElementById("product-modal-gst");
  productModalAvailable = document.getElementById("product-modal-available");
  productModalCancel = document.getElementById("product-modal-cancel");
  productModalSave = document.getElementById("product-modal-save");



  customerModal = document.getElementById("customer-modal");
  customerModalTitle = document.getElementById("customer-modal-title");
  customerModalName = document.getElementById("customer-modal-name");
  customerModalPhone = document.getElementById("customer-modal-phone");
  customerModalEmail = document.getElementById("customer-modal-email");
  customerModalCancel = document.getElementById("customer-modal-cancel");
  customerModalSave = document.getElementById("customer-modal-save");

  customerHistoryModal = document.getElementById("customer-history-modal");
  custHistoryName = document.getElementById("cust-history-name");
  custHistoryList = document.getElementById("cust-history-list");
  custHistoryClose = document.getElementById("cust-history-close");

  checkoutModal = document.getElementById("checkout-modal");
  checkoutModalAmount = document.getElementById("checkout-modal-amount");
  checkoutPaymentMode = document.getElementById("checkout-payment-mode");
  checkoutSplitBlock = document.getElementById("checkout-split-block");
  checkoutChangeBlock = document.getElementById("checkout-change-block");
  checkoutChangeRow = document.getElementById("checkout-change-row");
  checkoutChangeAmount = document.getElementById("checkout-change-amount");
  checkoutCashReceived = document.getElementById("checkout-cash-received");
  checkoutModalCancel = document.getElementById("checkout-modal-cancel");
  checkoutModalPrint = document.getElementById("checkout-modal-print");
  checkoutModalConfirm = document.getElementById("checkout-modal-confirm");

  splitCashAmount = document.getElementById("split-cash-amount");
  splitUpiAmount = document.getElementById("split-upi-amount");
  splitCardAmount = document.getElementById("split-card-amount");
  splitRemainingTotal = document.getElementById("split-remaining-total");

  // Receipt Preview
  receiptStoreName = document.getElementById("receipt-store-name");
  receiptStoreAddress = document.getElementById("receipt-store-address");
  receiptStorePhone = document.getElementById("receipt-store-phone");
  receiptStoreGstin = document.getElementById("receipt-store-gstin");
  receiptBillNumber = document.getElementById("receipt-bill-number");
  receiptDate = document.getElementById("receipt-date");
  receiptTable = document.getElementById("receipt-table");
  receiptCashier = document.getElementById("receipt-cashier");
  receiptCustomerRow = document.getElementById("receipt-customer-row");
  receiptCustomer = document.getElementById("receipt-customer");
  receiptItemsBody = document.getElementById("receipt-items-body");
  receiptSubtotal = document.getElementById("receipt-subtotal");
  receiptDiscount = document.getElementById("receipt-discount");
  receiptService = document.getElementById("receipt-service");
  receiptTax = document.getElementById("receipt-tax");
  receiptTotal = document.getElementById("receipt-total");
  receiptFooterMsg = document.getElementById("receipt-footer-msg");

  // Cancellation Modal
  cancelModal = document.getElementById("cancel-modal");
  cancelModalTitle = document.getElementById("cancel-modal-title");
  cancelModalItemName = document.getElementById("cancel-modal-item-name");
  cancelQtyGroup = document.getElementById("cancel-qty-group");
  cancelQtyInput = document.getElementById("cancel-qty-input");
  cancelReasonSelect = document.getElementById("cancel-reason-select");
  cancelReasonTextGroup = document.getElementById("cancel-reason-text-group");
  cancelReasonText = document.getElementById("cancel-reason-text");
  cancelModalCancelBtn = document.getElementById("cancel-modal-cancel-btn");
  cancelModalSubmitBtn = document.getElementById("cancel-modal-submit-btn");
  cancelOrderBtn = document.getElementById("cancel-order-btn");
}

function setupEventListeners() {
  // Login
  loginForm.addEventListener("submit", async (e) => {
    e.preventDefault();
    loginError.textContent = "";
    const attemptedUsername = usernameInput.value.trim();
    const attemptedPassword = passwordInput.value;

    try {
      const user = await invoke("login", { username: attemptedUsername, password: attemptedPassword });
      handleLoginSuccess(user);
    } catch (err) {
      loginError.textContent = typeof err === "string" ? err : "Login failed. Please try again.";
      console.error("[MealDesk] Login error:", err);
    }
  });

  // Navigation Panel Routing
  navItems.forEach(item => {
    item.addEventListener("click", () => {
      switchPanel(item.dataset.panel);
    });
  });

  document.getElementById("logout-btn").addEventListener("click", handleLogout);

  // Billing Operations
  productSearchInput.addEventListener("input", filterProducts);
  cartDiscountInput.addEventListener("input", renderCart);
  cartServiceInput.addEventListener("input", renderCart);
  cartCustomerSelect.addEventListener("change", updateSelectColors);
  cartTableSelect.addEventListener("change", updateSelectColors);
  clearCartBtn.addEventListener("click", () => {
    cart = [];
    activeOrderId = null;
    activeOrderStatus = null;
    cartTableSelect.value = "";
    cartCustomerSelect.value = "";
    renderCart();
    updateSelectColors();
  });

  // Checkout modal
  checkoutBtn.addEventListener("click", openCheckoutScreen);
  checkoutModalCancel.addEventListener("click", () => checkoutModal.classList.add("hidden"));
  checkoutPaymentMode.addEventListener("change", handlePaymentModeChange);
  checkoutCashReceived.addEventListener("input", calculateChangeAmount);
  checkoutModalPrint.addEventListener("click", () => window.print());
  checkoutModalConfirm.addEventListener("click", finalizeTransaction);

  // KOT button listener
  document.getElementById("kot-btn").addEventListener("click", sendKot);

  // Cancellation Event Listeners
  cancelOrderBtn.addEventListener("click", () => openCancellationModal({ type: 'order' }));
  cancelModalCancelBtn.addEventListener("click", () => cancelModal.classList.add("hidden"));
  cancelReasonSelect.addEventListener("change", () => {
    if (cancelReasonSelect.value === "Other") {
      cancelReasonTextGroup.classList.remove("hidden");
      cancelReasonText.required = true;
    } else {
      cancelReasonTextGroup.classList.add("hidden");
      cancelReasonText.required = false;
    }
  });
  cancelModalSubmitBtn.addEventListener("click", submitCancellation);

  // Set default printing layout class
  document.body.classList.add("print-receipt-mode");

  [splitCashAmount, splitUpiAmount, splitCardAmount].forEach(input => {
    input.addEventListener("input", calculateSplitPortions);
  });

  // Table transfers/merges
  tableTransferBtn.addEventListener("click", () => openTableActionModal("transfer"));
  tableMergeBtn.addEventListener("click", () => openTableActionModal("merge"));
  tableActionCancel.addEventListener("click", () => tablesActionModal.classList.add("hidden"));
  tableActionSubmit.addEventListener("click", submitTableAction);

  // Menu Category Modals
  addCategoryBtn.addEventListener("click", () => openCategoryModal(null));
  categoryModalCancel.addEventListener("click", () => categoryModal.classList.add("hidden"));
  categoryModalSave.addEventListener("click", saveCategory);

  // Menu Product Modals
  addProductBtn.addEventListener("click", () => openProductModal(null));
  productModalCancel.addEventListener("click", () => productModal.classList.add("hidden"));
  productModalSave.addEventListener("click", saveProduct);



  // Customer Management
  addCustomerBtn.addEventListener("click", () => openCustomerModal(null));
  customerModalCancel.addEventListener("click", () => customerModal.classList.add("hidden"));
  customerModalSave.addEventListener("click", saveCustomer);
  custHistoryClose.addEventListener("click", () => customerHistoryModal.classList.add("hidden"));

  // Reports
  generateReportBtn.addEventListener("click", generateReport);

  // Settings
  settingsRestForm.addEventListener("submit", saveSettings);
  settingsThemeSelect.addEventListener("change", toggleThemePreference);
  settingsGstDefault.addEventListener("change", () => {
    localStorage.setItem("gst_default", settingsGstDefault.value);
  });
  settingsPrinterSelect.addEventListener("change", () => {
    localStorage.setItem("printer_pref", settingsPrinterSelect.value);
  });

  // Backup & Restore handlers
  settingsBtnBackup.addEventListener("click", performBackup);
  settingsBtnRestore.addEventListener("click", performRestore);

  // Item cooking notes
  notesModalCancel.addEventListener("click", () => itemNotesModal.classList.add("hidden"));
  notesModalSave.addEventListener("click", saveItemNotes);

  // Modal final success closure
  modalCloseBtn.addEventListener("click", () => successModal.classList.add("hidden"));
}

function startClock() {
  setInterval(() => {
    const now = new Date();
    appClock.textContent = now.toLocaleTimeString([], { hour: '2-digit', minute: '2-digit', second: '2-digit' }) + 
      " | " + now.toLocaleDateString([], { day: '2-digit', month: 'short' });
  }, 1000);
}

function formatIndianDate(dateString) {
  const d = new Date(dateString);
  const day = String(d.getDate()).padStart(2, '0');
  const month = String(d.getMonth() + 1).padStart(2, '0');
  const year = d.getFullYear();
  return `${day}/${month}/${year}`;
}

function convertIndianToIso(dateStr) {
  const parts = dateStr.trim().split('/');
  if (parts.length === 3) {
    const day = parts[0].padStart(2, '0');
    const month = parts[1].padStart(2, '0');
    const year = parts[2];
    return `${year}-${month}-${day}`;
  }
  return dateStr;
}

function updateSelectColors() {
  if (cartCustomerSelect && cartCustomerSelect.value) {
    cartCustomerSelect.classList.add("has-selection");
  } else if (cartCustomerSelect) {
    cartCustomerSelect.classList.remove("has-selection");
  }

  if (cartTableSelect && cartTableSelect.value) {
    cartTableSelect.classList.add("has-selection");
  } else if (cartTableSelect) {
    cartTableSelect.classList.remove("has-selection");
  }
}

function loadSavedPreferences() {
  const theme = localStorage.getItem("theme_pref") || "dark";
  settingsThemeSelect.value = theme;
  if (theme === "light") {
    document.body.classList.add("light-theme");
  }

  const gst = localStorage.getItem("gst_default") || "18";
  settingsGstDefault.value = gst;

  const printer = localStorage.getItem("printer_pref") || "simulated";
  settingsPrinterSelect.value = printer;
}

function toggleThemePreference() {
  const theme = settingsThemeSelect.value;
  localStorage.setItem("theme_pref", theme);
  if (theme === "light") {
    document.body.classList.add("light-theme");
  } else {
    document.body.classList.remove("light-theme");
  }
}

// Backup & Restore
async function performBackup() {
  const path = settingsBackupPath.value.trim();
  if (!path) {
    alert("Please enter a valid directory path for the backup target");
    return;
  }
  try {
    await invoke("backup_db", { targetPath: path });
    modalTitle.textContent = "Backup Complete";
    modalMsg.textContent = `Offline database backup successfully created in target directory: ${path}`;
    successModal.classList.remove("hidden");
  } catch (err) {
    alert("Backup failed: " + err);
  }
}

async function performRestore() {
  const path = settingsRestorePath.value.trim();
  if (!path) {
    alert("Please enter a valid source backup file path");
    return;
  }
  try {
    await invoke("restore_db", { sourcePath: path });
    
    // Reload state after restoration
    restaurantInfo = await invoke("get_restaurant_info");
    restaurantNameHeader.textContent = restaurantInfo.name;
    categories = await invoke("get_categories");
    renderCategories();
    await loadCustomers();
    await loadTablesDropdown();

    modalTitle.textContent = "Database Restored";
    modalMsg.textContent = "Your local SQLite database has been successfully replaced with the backup file.";
    successModal.classList.remove("hidden");
  } catch (err) {
    alert("Restore failed: " + err);
  }
}

// Router & State Loads
async function handleLoginSuccess(user) {
  currentUser = user;
  localStorage.setItem("mealdesk_user", JSON.stringify(user));
  displayUserName.textContent = user.username.toUpperCase();

  // Set default report dates to current month
  const now = new Date();
  const firstDay = new Date(now.getFullYear(), now.getMonth(), 1);
  reportFromDate.value = formatIndianDate(firstDay);
  reportToDate.value = formatIndianDate(now);

  // Load backend details
  try {
    restaurantInfo = await invoke("get_restaurant_info");
    restaurantNameHeader.textContent = restaurantInfo.name;
    
    await loadCustomers();
    await loadTablesDropdown();

    categories = await invoke("get_categories");
    renderCategories();
  } catch (err) {
    console.error("Error loading application state details", err);
  }

  loginScreen.classList.add("hidden");
  appScreen.classList.remove("hidden");
  await switchPanel("pos");
}

function handleLogout() {
  currentUser = null;
  localStorage.removeItem("mealdesk_user");
  usernameInput.value = "";
  passwordInput.value = "";
  loginError.textContent = "";
  cart = [];
  activeOrderId = null;
  renderCart();
  
  appScreen.classList.add("hidden");
  loginScreen.classList.remove("hidden");
}

async function loadCustomers() {
  try {
    customers = await invoke("get_customers");
    cartCustomerSelect.innerHTML = `<option value="">Walk-in Customer</option>`;
    customers.forEach(c => {
      cartCustomerSelect.innerHTML += `<option value="${c.id}">${c.name} (${c.phone})</option>`;
    });
  } catch (err) {
    console.error(err);
  }
}

async function loadTablesDropdown() {
  try {
    const dbTables = await invoke("get_tables");
    cartTableSelect.innerHTML = `<option value="">No Table</option>`;
    dbTables.forEach(t => {
      if (!t.merged_into) {
        cartTableSelect.innerHTML += `<option value="${t.id}">${t.name}</option>`;
      }
    });
  } catch (err) {
    console.error(err);
  }
}

// Switching View Panels
async function switchPanel(panelId) {
  panels.forEach(p => p.classList.add("hidden"));
  const target = document.getElementById(`panel-${panelId}`);
  if (target) {
    target.classList.remove("hidden");
  }

  // Update active nav highlight
  navItems.forEach(nav => {
    if (nav.dataset.panel === panelId) {
      nav.classList.add("active");
    } else {
      nav.classList.remove("active");
    }
  });

  try {
    if (panelId === "pos") {
      activePanelTitle.textContent = "Billing POS";
    } else if (panelId === "tables") {
      activePanelTitle.textContent = "Restaurant Tables";
      await renderTables();
    } else if (panelId === "kitchen") {
      activePanelTitle.textContent = "Kitchen Live Display";
      await loadKots();
    } else if (panelId === "menu-editor") {
      activePanelTitle.textContent = "Restaurant Menu Management";
      await renderCategoryEditor();
    } else if (panelId === "customers") {
      activePanelTitle.textContent = "Customer Profiles Directory";
      await renderCustomersList();
    } else if (panelId === "reports") {
      activePanelTitle.textContent = "Sales Performance Metrics";
      await generateReport();
    } else if (panelId === "settings") {
      activePanelTitle.textContent = "Application Settings";
      loadSettingsForm();
    }
  } catch (err) {
    console.error(`[MealDesk] Error loading panel '${panelId}':`, err);
  }
}

// 1. POS Operations
function renderCategories() {
  posCategoriesContainer.innerHTML = "";
  if (categories.length === 0) {
    posCategoriesContainer.innerHTML = `<span style="color:var(--text-muted)">No categories found</span>`;
    return;
  }

  // "All Items" tab — default selected
  const allTab = document.createElement("div");
  allTab.className = "category-tab active";
  allTab.textContent = "All Items";
  allTab.dataset.id = "all";
  allTab.addEventListener("click", () => {
    document.querySelectorAll(".category-tab").forEach(t => t.classList.remove("active"));
    allTab.classList.add("active");
    activeCategoryId = null;
    loadAllProducts();
  });
  posCategoriesContainer.appendChild(allTab);

  // Load all products by default
  activeCategoryId = null;
  loadAllProducts();

  categories.forEach((cat) => {
    const tab = document.createElement("div");
    tab.className = "category-tab";
    tab.textContent = cat.name;
    tab.dataset.id = cat.id;

    tab.addEventListener("click", () => {
      document.querySelectorAll(".category-tab").forEach(t => t.classList.remove("active"));
      tab.classList.add("active");
      activeCategoryId = cat.id;
      loadProducts(cat.id);
    });

    posCategoriesContainer.appendChild(tab);
  });
}

async function loadAllProducts() {
  try {
    const allProducts = [];
    for (const cat of categories) {
      const catProducts = await invoke("get_products_by_category", { categoryId: cat.id });
      allProducts.push(...catProducts);
    }
    products = allProducts;
    renderProducts(products);
  } catch (err) {
    console.error(err);
  }
}

async function loadProducts(categoryId) {
  try {
    products = await invoke("get_products_by_category", { categoryId });
    renderProducts(products);
  } catch (err) {
    console.error(err);
  }
}

function renderProducts(productsList) {
  posProductsContainer.innerHTML = "";
  if (productsList.length === 0) {
    posProductsContainer.innerHTML = `<span style="grid-column: 1/-1; text-align: center; color: var(--text-muted); padding: 40px 0;">No available items in this category</span>`;
    return;
  }

  productsList.forEach(prod => {
    if (prod.is_available) {
      const card = document.createElement("div");
      card.className = "product-card";
      card.innerHTML = `
        <div class="product-name">${prod.name}</div>
        <div class="product-meta">
          <div class="product-price">₹${prod.price.toFixed(2)}</div>
          <div class="product-gst">GST ${prod.gst_rate}%</div>
        </div>
      `;
      card.addEventListener("click", () => addToCart(prod));
      posProductsContainer.appendChild(card);
    }
  });
}

function filterProducts() {
  const query = productSearchInput.value.toLowerCase().trim();
  if (!query) {
    renderProducts(products);
    return;
  }
  const filtered = products.filter(p => p.name.toLowerCase().includes(query));
  renderProducts(filtered);
}

function addToCart(product) {
  if (activeOrderStatus === "Billed" || activeOrderStatus === "Completed") {
    alert("Order is billed and cannot be modified.");
    return;
  }
  const existing = cart.find(item => item.product.id === product.id && !item.kot_id);
  if (existing) {
    existing.quantity += 1;
  } else {
    cart.push({
      product: product,
      quantity: 1,
      notes: ""
    });
  }
  renderCart();
}

function updateCartQty(productId, change) {
  if (activeOrderStatus === "Billed" || activeOrderStatus === "Completed") {
    alert("Order is billed and cannot be modified.");
    return;
  }
  const idx = cart.findIndex(item => item.product.id === productId && !item.kot_id);
  if (idx === -1) return;

  cart[idx].quantity += change;
  if (cart[idx].quantity <= 0) {
    cart.splice(idx, 1);
  }
  renderCart();
}

function renderCart() {
  // Update button visibility inline (moved to checkout area)
  if (activeOrderId) {
    cancelOrderBtn.classList.remove("hidden");
    clearCartBtn.classList.add("hidden");
  } else {
    cancelOrderBtn.classList.add("hidden");
    clearCartBtn.classList.remove("hidden");
  }

  cartItemsList.innerHTML = "";
  
  const isLocked = activeOrderStatus === "Billed" || activeOrderStatus === "Completed";
  
  cartDiscountInput.disabled = isLocked;
  cartServiceInput.disabled = isLocked;
  cartCustomerSelect.disabled = isLocked;
  cartTableSelect.disabled = isLocked;
  
  if (isLocked) {
    document.getElementById("kot-btn").classList.add("hidden");
  } else {
    document.getElementById("kot-btn").classList.remove("hidden");
  }

  if (cart.length === 0) {
    cartItemsList.innerHTML = `
      <div class="cart-empty">
        <span>🛒</span>
        <span>Order is empty</span>
      </div>
    `;
    cartSubtotalEl.textContent = "₹0.00";
    cartTaxEl.textContent = "₹0.00";
    cartTotalEl.textContent = "₹0.00";
    return;
  }

  let subtotal = 0;
  let totalTax = 0;

  const discountPercent = parseFloat(cartDiscountInput.value) || 0;
  const servicePercent = parseFloat(cartServiceInput.value) || 0;

  cart.forEach((item, index) => {
    const effectiveQty = item.quantity - (item.cancelled_quantity || 0);
    const itemSubtotal = item.product.price * effectiveQty;
    const itemTax = itemSubtotal * (item.product.gst_rate / 100);
    
    subtotal += itemSubtotal;
    totalTax += itemTax;

    const div = document.createElement("div");
    if (item.kot_id) {
      // Locked item
      div.className = "cart-item locked";
      div.innerHTML = `
        <div class="cart-item-details" style="cursor: pointer;">
          <div class="cart-item-name" style="font-weight: 600;">
            ${item.product.name} <span class="badge locked-badge" style="background: var(--bg-hover); font-size: 0.7rem; padding: 2px 6px; border-radius: 4px; color: var(--accent);">KOT #${item.kot_id}</span>
          </div>
          <div class="cart-item-price">
            ₹${item.product.price.toFixed(2)} x ${effectiveQty}
            ${item.cancelled_quantity > 0 ? `<span class="cancelled-text" style="color: var(--danger); font-size: 0.8rem; margin-left: 5px;">(Cancelled ${item.cancelled_quantity})</span>` : ''}
          </div>
          ${item.notes ? `<div style="font-size:0.75rem; color:var(--warning); font-style:italic;">📝 ${item.notes}</div>` : ""}
        </div>
        <div class="cart-item-control">
          ${effectiveQty > 0 
            ? `<button class="qty-btn cancel-btn" style="background: rgba(239, 68, 68, 0.1); color: var(--danger); border-color: rgba(239, 68, 68, 0.2); width: auto; padding: 2px 8px; border-radius: 4px; font-size: 0.75rem;" data-index="${index}">Cancel</button>` 
            : `<span class="cancelled-label" style="color: var(--danger); font-size: 0.75rem; font-weight: bold;">Fully Cancelled</span>`}
        </div>
      `;
      
      const cancelBtn = div.querySelector(".cancel-btn");
      if (cancelBtn) {
        cancelBtn.addEventListener("click", (e) => {
          e.stopPropagation();
          openCancellationModal({ type: 'item', itemIndex: index });
        });
      }
    } else {
      // Unlocked (editable) item
      div.className = "cart-item" + (isLocked ? " locked" : "");
      div.innerHTML = `
        <div class="cart-item-details" style="cursor: pointer;">
          <div class="cart-item-name">${item.product.name}</div>
          <div class="cart-item-price">₹${item.product.price.toFixed(2)} x ${item.quantity}</div>
          ${item.notes ? `<div style="font-size:0.75rem; color:var(--warning); font-style:italic;">📝 ${item.notes}</div>` : ""}
        </div>
        <div class="cart-item-control">
          ${isLocked 
            ? `<span class="cart-item-qty" style="padding-right: 10px;">Qty: ${item.quantity}</span>`
            : `
              <button class="qty-btn minus">-</button>
              <span class="cart-item-qty">${item.quantity}</span>
              <button class="qty-btn plus">+</button>
            `
          }
        </div>
      `;
      if (!isLocked) {
        div.querySelector(".minus").addEventListener("click", () => updateCartQty(item.product.id, -1));
        div.querySelector(".plus").addEventListener("click", () => updateCartQty(item.product.id, 1));
      }
    }

    div.querySelector(".cart-item-details").addEventListener("click", () => {
      if (isLocked) return;
      activeNotesItemIndex = index;
      notesProductName.textContent = item.product.name;
      itemNotesText.value = item.notes;
      itemNotesModal.classList.remove("hidden");
    });

    cartItemsList.appendChild(div);
  });

  const discountAmount = subtotal * (discountPercent / 100);
  const serviceAmount = subtotal * (servicePercent / 100);
  const taxableSubtotal = subtotal - discountAmount + serviceAmount;

  const finalTax = taxableSubtotal * (totalTax / subtotal || 0);
  const totalRaw = taxableSubtotal + finalTax;
  const totalRounded = Math.round(totalRaw);

  cartSubtotalEl.textContent = `₹${subtotal.toFixed(2)}`;
  cartTaxEl.textContent = `₹${finalTax.toFixed(2)}`;
  cartTotalEl.textContent = `₹${totalRounded.toFixed(2)}`;
}

function saveItemNotes() {
  if (activeNotesItemIndex !== null && activeNotesItemIndex !== undefined) {
    cart[activeNotesItemIndex].notes = itemNotesText.value.trim();
    renderCart();
  }
  itemNotesModal.classList.add("hidden");
}

// Billing Holds
async function resumeOrder(orderId) {
  try {
    const orderData = await invoke("get_order", { orderId });
    
    cart = orderData.items.map(it => ({
      id: it.id,
      product: {
        id: it.product_id,
        name: it.name,
        price: it.price,
        gst_rate: it.gst_rate,
        is_available: true
      },
      quantity: it.quantity,
      cancelled_quantity: it.cancelled_quantity,
      notes: it.notes || "",
      kot_id: it.kot_id
    }));

    cartDiscountInput.value = orderData.header.discount;
    cartServiceInput.value = orderData.header.service_charge;
    cartCustomerSelect.value = orderData.header.customer_id || "";
    cartTableSelect.value = orderData.header.table_id || "";

    activeOrderId = orderId;
    activeOrderStatus = orderData.header.status;
    updateSelectColors();
    renderCart();
  } catch (err) {
    alert("Error resuming order: " + err);
  }
}

// Checkout Screens
async function openCheckoutScreen() {
  if (cart.length === 0) return;

  const subtotal = parseFloat(cartSubtotalEl.textContent.replace('₹','')) || 0;
  const discountPercent = parseFloat(cartDiscountInput.value) || 0;
  const servicePercent = parseFloat(cartServiceInput.value) || 0;
  const discountAmount = subtotal * (discountPercent / 100);
  const serviceAmount = subtotal * (servicePercent / 100);
  const finalTax = parseFloat(cartTaxEl.textContent.replace('₹','')) || 0;
  const finalTotal = parseFloat(cartTotalEl.textContent.replace('₹','')) || 0;

  const tableIdVal = cartTableSelect.value ? parseInt(cartTableSelect.value) : null;
  const custIdVal = cartCustomerSelect.value ? parseInt(cartCustomerSelect.value) : null;

  const orderItemsInput = cart.map(item => ({
    id: item.id || null,
    product_id: item.product.id,
    name: item.product.name,
    quantity: item.quantity,
    price: item.product.price,
    gst_rate: item.product.gst_rate,
    notes: item.notes || null
  }));

  try {
    let orderId = activeOrderId;
    
    // Save or update the order first
    if (activeOrderId) {
      await invoke("update_order", {
        orderId: activeOrderId,
        tableId: tableIdVal,
        customerId: custIdVal,
        notes: "",
        items: orderItemsInput,
        createdAt: new Date().toISOString(),
        username: currentUser.username
      });
    } else {
      orderId = await invoke("create_order", {
        tableId: tableIdVal,
        customerId: custIdVal,
        notes: "",
        items: orderItemsInput,
        createdAt: new Date().toISOString(),
        username: currentUser.username
      });
      activeOrderId = orderId;
    }

    // Generate the bill on the backend
    const orderHeader = await invoke("generate_bill", {
      orderId: orderId,
      discount: discountPercent,
      serviceCharge: servicePercent,
      username: currentUser.username
    });

    activeOrderStatus = "Billed";

    // Modals inputs reset using totals from backend
    checkoutModalAmount.textContent = `₹${orderHeader.total.toFixed(2)}`;
    checkoutPaymentMode.value = "Cash";
    checkoutCashReceived.value = "";
    checkoutChangeAmount.textContent = "₹0.00";
    checkoutSplitBlock.classList.add("hidden");
    checkoutChangeBlock.classList.remove("hidden");
    checkoutChangeRow.classList.remove("hidden");
    checkoutModalConfirm.disabled = false;

    splitCashAmount.value = 0;
    splitUpiAmount.value = 0;
    splitCardAmount.value = 0;
    splitRemainingTotal.textContent = `₹${orderHeader.total.toFixed(2)}`;

    // Populate print template
    receiptStoreName.textContent = (restaurantInfo && restaurantInfo.name) ? restaurantInfo.name.toUpperCase() : "MEALDESK BISTRO";
    receiptStoreAddress.textContent = (restaurantInfo && restaurantInfo.address) ? restaurantInfo.address : "";
    receiptStorePhone.textContent = (restaurantInfo && restaurantInfo.phone) ? `Ph: ${restaurantInfo.phone}` : "";
    receiptStoreGstin.textContent = (restaurantInfo && restaurantInfo.gstin) ? `GSTIN: ${restaurantInfo.gstin}` : "";
    receiptBillNumber.textContent = `#${orderHeader.id}`;
    
    const now = new Date(orderHeader.created_at);
    receiptDate.textContent = now.toLocaleDateString([], { day: '2-digit', month: 'short', year: 'numeric' }) + " " + now.toLocaleTimeString([], { hour: '2-digit', minute: '2-digit' });
    
    receiptTable.textContent = orderHeader.table_name || "No Table";
    receiptCashier.textContent = currentUser ? currentUser.username : "Cashier";

    if (orderHeader.customer_name) {
      receiptCustomerRow.classList.remove("hidden");
      receiptCustomer.textContent = orderHeader.customer_name;
    } else {
      receiptCustomerRow.classList.add("hidden");
    }

    // Populate items in the receipt
    receiptItemsBody.innerHTML = "";
    
    const consolidated = {};
    cart.forEach(item => {
      const effectiveQty = item.quantity - (item.cancelled_quantity || 0);
      if (effectiveQty <= 0) return;
      
      const pid = item.product.id;
      if (!consolidated[pid]) {
        consolidated[pid] = {
          name: item.product.name,
          price: item.product.price,
          quantity: 0,
          notes: []
        };
      }
      consolidated[pid].quantity += effectiveQty;
      if (item.notes) {
        consolidated[pid].notes.push(item.notes);
      }
    });

    Object.values(consolidated).forEach(item => {
      const tr = document.createElement("tr");
      const notesStr = item.notes.length > 0 ? `<div style="font-size:0.6rem; color:#777; font-style:italic;">* ${item.notes.join(', ')}</div>` : "";
      tr.innerHTML = `
        <td style="padding: 4px 0;">
          ${item.name}
          ${notesStr}
        </td>
        <td style="text-align: center; padding: 4px 0;">${item.quantity}</td>
        <td style="text-align: right; padding: 4px 0;">₹${(item.price * item.quantity).toFixed(2)}</td>
      `;
      receiptItemsBody.appendChild(tr);
    });

    const billDiscountAmount = orderHeader.subtotal * (orderHeader.discount / 100);
    const billServiceAmount = orderHeader.subtotal * (orderHeader.service_charge / 100);

    receiptSubtotal.textContent = `₹${orderHeader.subtotal.toFixed(2)}`;
    receiptDiscount.textContent = `-₹${billDiscountAmount.toFixed(2)}`;
    receiptService.textContent = `₹${billServiceAmount.toFixed(2)}`;
    receiptTax.textContent = `₹${orderHeader.tax.toFixed(2)}`;
    receiptTotal.textContent = `₹${orderHeader.total.toFixed(2)}`;
    receiptFooterMsg.textContent = (restaurantInfo && restaurantInfo.receipt_footer) ? restaurantInfo.receipt_footer : "Thank you for dining with us!";

    checkoutModal.classList.remove("hidden");
    
    renderCart();
  } catch (err) {
    alert("Error opening checkout screen: " + err);
    console.error(err);
  }
}

function handlePaymentModeChange() {
  const mode = checkoutPaymentMode.value;
  const originalTotal = parseFloat(checkoutModalAmount.textContent.replace('₹','')) || 0;

  if (mode === "NC") {
    receiptTotal.textContent = "₹0.00";
  } else {
    receiptTotal.textContent = `₹${originalTotal.toFixed(2)}`;
  }

  if (mode === "Mixed") {
    checkoutSplitBlock.classList.remove("hidden");
    checkoutChangeBlock.classList.add("hidden");
    checkoutChangeRow.classList.add("hidden");
    calculateSplitPortions();
  } else if (mode === "Cash") {
    checkoutSplitBlock.classList.add("hidden");
    checkoutChangeBlock.classList.remove("hidden");
    checkoutChangeRow.classList.remove("hidden");
    checkoutModalConfirm.disabled = false;
    calculateChangeAmount();
  } else {
    checkoutSplitBlock.classList.add("hidden");
    checkoutChangeBlock.classList.add("hidden");
    checkoutChangeRow.classList.add("hidden");
    checkoutModalConfirm.disabled = false;
  }
}

function calculateChangeAmount() {
  const payable = parseFloat(checkoutModalAmount.textContent.replace('₹','')) || 0;
  const received = parseFloat(checkoutCashReceived.value) || 0;
  if (received >= payable) {
    checkoutChangeAmount.textContent = `₹${(received - payable).toFixed(2)}`;
  } else {
    checkoutChangeAmount.textContent = "₹0.00";
  }
}

function calculateSplitPortions() {
  const payable = parseFloat(checkoutModalAmount.textContent.replace('₹','')) || 0;
  const cash = parseFloat(splitCashAmount.value) || 0;
  const upi = parseFloat(splitUpiAmount.value) || 0;
  const card = parseFloat(splitCardAmount.value) || 0;

  const sum = cash + upi + card;
  const remaining = payable - sum;

  if (Math.abs(remaining) < 0.01) {
    splitRemainingTotal.textContent = "₹0.00";
    splitRemainingTotal.style.color = "var(--success)";
    checkoutModalConfirm.disabled = false;
  } else {
    splitRemainingTotal.textContent = `₹${remaining.toFixed(2)}`;
    splitRemainingTotal.style.color = "var(--danger)";
    checkoutModalConfirm.disabled = true;
  }
}

async function finalizeTransaction() {
  const paymentMode = checkoutPaymentMode.value;
  let paymentsList = [];
  const billTotalVal = parseFloat(checkoutModalAmount.textContent.replace('₹','')) || 0;

  if (paymentMode === "Cash") {
    const received = parseFloat(checkoutCashReceived.value) || billTotalVal;
    paymentsList.push({
      payment_method: "Cash",
      amount: received
    });
  } else if (paymentMode === "UPI") {
    paymentsList.push({
      payment_method: "UPI",
      amount: billTotalVal
    });
  } else if (paymentMode === "Card") {
    paymentsList.push({
      payment_method: "Card",
      amount: billTotalVal
    });
  } else if (paymentMode === "NC") {
    paymentsList.push({
      payment_method: "NC",
      amount: 0.0
    });
  } else if (paymentMode === "Mixed") {
    const cash = parseFloat(splitCashAmount.value) || 0;
    const upi = parseFloat(splitUpiAmount.value) || 0;
    const card = parseFloat(splitCardAmount.value) || 0;
    
    if (cash > 0) paymentsList.push({ payment_method: "Cash", amount: cash });
    if (upi > 0) paymentsList.push({ payment_method: "UPI", amount: upi });
    if (card > 0) paymentsList.push({ payment_method: "Card", amount: card });
  }

  try {
    await invoke("record_payments", {
      orderId: activeOrderId,
      payments: paymentsList,
      username: currentUser.username
    });

    const printerMode = localStorage.getItem("printer_pref") || "simulated";
    if (printerMode === "system") {
      window.print();
    }

    // Capture order ID before clearing state
    const completedOrderId = activeOrderId;

    checkoutModal.classList.add("hidden");
    cart = [];
    activeOrderId = null;
    activeOrderStatus = null;
    cartTableSelect.value = "";
    cartCustomerSelect.value = "";
    renderCart();
    updateSelectColors();
    
    await loadCustomers();
    
    modalTitle.textContent = "Bill Completed Successfully";
    modalMsg.innerHTML = `
      <strong>Invoice ID: #${completedOrderId}</strong><br/>
      Total billing amount: ₹${billTotalVal.toFixed(2)} (${paymentMode})<br/>
      Receipt printed successfully.
    `;
    successModal.classList.remove("hidden");
  } catch (err) {
    alert("Error completing transaction: " + err);
  }
}

// 2. Table Layout
async function renderTables() {
  tablesGrid.innerHTML = "";
  try {
    const dbTables = await invoke("get_tables");
    dbTables.forEach(t => {
      if (t.merged_into) return;

      const card = document.createElement("div");
      card.className = `table-card ${t.status.toLowerCase()}`;
      
      card.innerHTML = `
        <div class="table-number">${t.name}</div>
        <div class="table-status-label">${t.status}</div>

        ${t.current_order_total ? `<div style="font-size:0.85rem; font-weight:600; color:var(--accent);">₹${t.current_order_total.toFixed(2)}</div>` : ""}
      `;

      card.addEventListener("click", () => {
        if (t.current_order_id) {
          resumeOrder(t.current_order_id);
          switchPanel("pos");
        } else {
          cart = [];
          activeOrderId = null;
          cartTableSelect.value = t.id;
          updateSelectColors();
          renderCart();
          switchPanel("pos");
        }
      });
      tablesGrid.appendChild(card);
    });
  } catch (err) {
    console.error(err);
  }
}

async function openTableActionModal(type) {
  activeTableActionType = type;
  try {
    const dbTables = await invoke("get_tables");
    tableActionFromSelect.innerHTML = "";
    tableActionToSelect.innerHTML = "";
    
    if (type === "transfer") {
      tableActionTitle.textContent = "Transfer Active Bill";
      dbTables.forEach(t => {
        if ((t.status === "Occupied" || t.status === "Billed") && !t.merged_into) {
          tableActionFromSelect.innerHTML += `<option value="${t.id}">${t.name} (₹${t.current_order_total ? t.current_order_total.toFixed(2) : '0'})</option>`;
        }
      });
      dbTables.forEach(t => {
        if (t.status === "Free" && !t.merged_into) {
          tableActionToSelect.innerHTML += `<option value="${t.id}">${t.name}</option>`;
        }
      });
    } else {
      tableActionTitle.textContent = "Merge Tables";
      dbTables.forEach(t => {
        if (!t.merged_into) {
          tableActionFromSelect.innerHTML += `<option value="${t.id}">${t.name}</option>`;
        }
      });
      dbTables.forEach(t => {
        if (!t.merged_into) {
          tableActionToSelect.innerHTML += `<option value="${t.id}">${t.name}</option>`;
        }
      });
    }

    tablesActionModal.classList.remove("hidden");
  } catch (err) {
    console.error(err);
  }
}

async function submitTableAction() {
  const fromId = parseInt(tableActionFromSelect.value);
  const toId = parseInt(tableActionToSelect.value);
  if (!fromId || !toId || fromId === toId) return;

  try {
    if (activeTableActionType === "transfer") {
      await invoke("transfer_table", { fromTableId: fromId, toTableId: toId, username: currentUser.username });
      tablesActionModal.classList.add("hidden");
      renderTables();
    } else {
      await invoke("merge_tables", { sourceTableId: fromId, targetTableId: toId, username: currentUser.username });
      tablesActionModal.classList.add("hidden");
      renderTables();
    }
  } catch (err) {
    alert("Error performing table operation: " + err);
  }
}

// 3. Kitchen KOT displays
async function loadKots() {
  try {
    const kots = await invoke("get_active_kots");
    kitchenKotGrid.innerHTML = "";

    if (kots.length === 0) {
      kitchenKotGrid.innerHTML = `
        <div class="cart-empty" style="grid-column: 1/-1; height: 300px;">
          <span>🍳</span>
          <span>No active KOT tickets</span>
        </div>
      `;
      return;
    }

    kots.forEach(kot => {
      const card = document.createElement("div");
      card.className = `kot-card ${kot.status.toLowerCase()}`;
      
      const timeStr = new Date(kot.created_at).toLocaleTimeString([], { hour: '2-digit', minute: '2-digit' });
      
      let itemsHtml = "";
      kot.items.forEach(it => {
        const isCancelled = it.quantity < 0;
        const qtyDisplay = isCancelled ? Math.abs(it.quantity) : it.quantity;
        const nameDisplay = isCancelled ? `<del style="color: var(--danger); text-decoration: line-through;">${it.product_name} (CANCELLED)</del>` : it.product_name;
        itemsHtml += `
          <div class="kot-item-row" style="${isCancelled ? 'background-color: rgba(239, 68, 68, 0.08); padding: 4px; border-radius: 4px;' : ''}">
            <span class="kot-item-qty-name" style="${isCancelled ? 'color: var(--danger);' : ''}">${qtyDisplay} x ${nameDisplay}</span>
          </div>
          ${it.notes ? `<div class="kot-item-notes">Notes: ${it.notes}</div>` : ""}
        `;
      });

      let btnHtml = "";
      if (kot.status === "Pending") {
        btnHtml = `<button class="kot-btn prepare" style="flex: 1;">Start Preparing</button>`;
      } else if (kot.status === "Preparing") {
        btnHtml = `<button class="kot-btn ready" style="flex: 1;">Mark Ready</button>`;
      } else if (kot.status === "Ready") {
        btnHtml = `<button class="kot-btn complete" style="flex: 1;">Complete KOT</button>`;
      }
      
      // Add KOT print button in the actions row
      btnHtml += `<button class="kot-btn print-kot-action-btn" style="background: var(--bg-hover); max-width: 50px; flex: 0 0 50px;" title="Print KOT">🖨️</button>`;

      card.innerHTML = `
        <div class="kot-header">
          <div class="kot-title">KOT #${kot.id} [${kot.table_name || 'No Table'}]</div>
          <div class="kot-time">${timeStr}</div>
        </div>
        <div class="kot-items-list">${itemsHtml}</div>
        <div class="kot-actions">${btnHtml}</div>
      `;

      const actionBtn = card.querySelector(".kot-btn:not(.print-kot-action-btn)");
      if (actionBtn) {
        actionBtn.addEventListener("click", () => advanceKotState(kot.id, kot.status));
      }

      const printBtn = card.querySelector(".print-kot-action-btn");
      if (printBtn) {
        printBtn.addEventListener("click", () => printKotById(kot.id));
      }

      kitchenKotGrid.appendChild(card);
    });
  } catch (err) {
    console.error(err);
  }
}

async function advanceKotState(kotId, currentStatus) {
  let nextStatus = "Completed";
  if (currentStatus === "Pending") nextStatus = "Preparing";
  else if (currentStatus === "Preparing") nextStatus = "Ready";
  
  try {
    await invoke("update_kot_status", { kotId, status: nextStatus, username: currentUser.username });
    loadKots();
  } catch (err) {
    alert("Error updating KOT status: " + err);
  }
}

// 4. Menu CRUD Editor Actions
async function renderCategoryEditor() {
  menuEditorCategories.innerHTML = "";
  try {
    const list = await invoke("get_categories");
    list.forEach(c => {
      const item = document.createElement("div");
      item.className = "category-editor-item";
      item.innerHTML = `
        <span class="category-editor-name">${c.name}</span>
        <div style="display:flex; gap:6px;">
          <button class="quick-role-btn edit-cat-btn" style="padding: 2px 6px;">✏️</button>
          <button class="quick-role-btn del-cat-btn" style="padding: 2px 6px; color: var(--danger); border-color: rgba(239,68,68,0.3);">🗑️</button>
        </div>
      `;

      item.addEventListener("click", (e) => {
        if (e.target.tagName === "BUTTON") return;
        document.querySelectorAll(".category-editor-item").forEach(i => i.classList.remove("active"));
        item.classList.add("active");
        renderProductsEditorList(c.id);
      });

      item.querySelector(".edit-cat-btn").addEventListener("click", (e) => {
        e.stopPropagation();
        openCategoryModal(c);
      });

      item.querySelector(".del-cat-btn").addEventListener("click", async (e) => {
        e.stopPropagation();
        if (!confirm(`Delete category "${c.name}"? This cannot be undone.`)) return;
        try {
          await invoke("delete_category", { id: c.id });
          categories = await invoke("get_categories");
          renderCategories();
          renderCategoryEditor();
        } catch (err) {
          alert("Cannot delete: " + err);
        }
      });

      menuEditorCategories.appendChild(item);
    });

    if (list.length > 0) {
      menuEditorCategories.firstChild.classList.add("active");
      renderProductsEditorList(list[0].id);
    } else {
      menuEditorProductsList.innerHTML = `<tr><td colspan="5" style="text-align:center; color:var(--text-muted);">Please create a category first</td></tr>`;
    }
  } catch (err) {
    console.error(err);
  }
}

function openCategoryModal(category) {
  if (category) {
    categoryModalTitle.textContent = "Edit Category";
    categoryModalName.value = category.name;
    categoryModalDesc.value = category.description || "";
    activeCategoryEditId = category.id;
  } else {
    categoryModalTitle.textContent = "Add Category";
    categoryModalName.value = "";
    categoryModalDesc.value = "";
    activeCategoryEditId = null;
  }
  categoryModal.classList.remove("hidden");
}

async function saveCategory() {
  const name = categoryModalName.value.trim();
  const desc = categoryModalDesc.value.trim();
  if (!name) return;

  try {
    await invoke("upsert_category", {
      id: activeCategoryEditId,
      name,
      description: desc || null
    });
    categoryModal.classList.add("hidden");
    
    categories = await invoke("get_categories");
    renderCategories();
    renderCategoryEditor();
  } catch (err) {
    alert("Error saving category: " + err);
  }
}

async function renderProductsEditorList(categoryId) {
  menuEditorProductsList.innerHTML = "";
  try {
    const list = await invoke("get_products_by_category", { categoryId });
    
    if (list.length === 0) {
      menuEditorProductsList.innerHTML = `<tr><td colspan="5" style="text-align:center; color:var(--text-muted); padding:20px;">No products in this category</td></tr>`;
      return;
    }

    list.forEach(p => {
      const tr = document.createElement("tr");
      tr.innerHTML = `
        <td style="padding:10px;">${p.name}</td>
        <td style="padding:10px;">₹${p.price.toFixed(2)}</td>
        <td style="padding:10px;">GST ${p.gst_rate}%</td>
        <td style="padding:10px;">
          <span class="status-badge ${p.is_available ? 'in-stock' : 'out-of-stock'}">
            ${p.is_available ? 'Available' : 'Unavailable'}
          </span>
        </td>
        <td style="padding:10px; text-align:right;">
          <button class="quick-role-btn edit-p-btn">Edit</button>
          <button class="quick-role-btn del-p-btn" style="margin-left:4px; color: var(--danger); border-color: rgba(239,68,68,0.3);">Delete</button>
        </td>
      `;
      tr.querySelector(".edit-p-btn").addEventListener("click", () => openProductModal(p));
      tr.querySelector(".del-p-btn").addEventListener("click", async () => {
        if (!confirm(`Delete "${p.name}"? This cannot be undone.`)) return;
        try {
          await invoke("delete_product", { id: p.id });
          renderProductsEditorList(categoryId);
        } catch (err) {
          alert("Cannot delete: " + err);
        }
      });
      menuEditorProductsList.appendChild(tr);
    });
  } catch (err) {
    console.error(err);
  }
}

async function openProductModal(product) {
  productModalCategory.innerHTML = "";
  categories.forEach(c => {
    productModalCategory.innerHTML += `<option value="${c.id}">${c.name}</option>`;
  });

  if (product) {
    productModalTitle.textContent = "Edit Product";
    productModalCategory.value = product.category_id;
    productModalName.value = product.name;
    productModalPrice.value = product.price;
    productModalGst.value = Math.round(product.gst_rate).toString();
    productModalAvailable.checked = product.is_available;
    activeProductEditId = product.id;
  } else {
    productModalTitle.textContent = "Add Product";
    productModalCategory.value = activeCategoryId || "";
    productModalName.value = "";
    productModalPrice.value = "";
    productModalGst.value = localStorage.getItem("gst_default") || "18";
    productModalAvailable.checked = true;
    activeProductEditId = null;
  }
  productModal.classList.remove("hidden");
}

async function saveProduct() {
  const catId = parseInt(productModalCategory.value);
  const name = productModalName.value.trim();
  const price = parseFloat(productModalPrice.value) || 0;
  const gst = parseFloat(productModalGst.value) || 0;
  const isAvailable = productModalAvailable.checked;

  if (!name || !price) return;

  try {
    await invoke("upsert_product", {
      id: activeProductEditId,
      categoryId: catId,
      name,
      price,
      gstRate: gst,
      isAvailable
    });
    productModal.classList.add("hidden");
    
    renderProductsEditorList(catId);
    if (activeCategoryId === catId) {
      loadProducts(catId);
    }
  } catch (err) {
    alert("Error saving product: " + err);
  }
}



// 6. Customer Profiles
async function renderCustomersList() {
  customersList.innerHTML = "";
  try {
    const list = await invoke("get_customers");
    if (list.length === 0) {
      customersList.innerHTML = `<tr><td colspan="5" style="text-align:center; color:var(--text-muted); padding:30px;">No registered customers</td></tr>`;
      return;
    }

    list.forEach(c => {
      const tr = document.createElement("tr");
      tr.innerHTML = `
        <td style="padding:12px 16px;">${c.name}</td>
        <td style="padding:12px 16px;">${c.phone}</td>
        <td style="padding:12px 16px;">${c.email || '-'}</td>
        <td style="padding:12px 16px; font-weight:600; color:var(--primary);">${c.loyalty_points} pts</td>
        <td style="padding:12px 16px; text-align:right;">
          <button class="quick-role-btn history-cust-btn" style="margin-right:4px;">History</button>
          <button class="quick-role-btn edit-cust-btn">Edit</button>
        </td>
      `;
      tr.querySelector(".edit-cust-btn").addEventListener("click", () => openCustomerModal(c));
      tr.querySelector(".history-cust-btn").addEventListener("click", () => openCustomerHistory(c));
      customersList.appendChild(tr);
    });
  } catch (err) {
    console.error(err);
  }
}

function openCustomerModal(customer) {
  if (customer) {
    customerModalTitle.textContent = "Edit Customer Details";
    customerModalName.value = customer.name;
    customerModalPhone.value = customer.phone;
    customerModalEmail.value = customer.email || "";
    activeCustomerEditId = customer.id;
  } else {
    customerModalTitle.textContent = "Register Customer";
    customerModalName.value = "";
    customerModalPhone.value = "";
    customerModalEmail.value = "";
    activeCustomerEditId = null;
  }
  customerModal.classList.remove("hidden");
}

async function saveCustomer() {
  const name = customerModalName.value.trim();
  const phone = customerModalPhone.value.trim();
  const email = customerModalEmail.value.trim();
  
  if (!name || !phone) return;

  try {
    await invoke("upsert_customer", {
      id: activeCustomerEditId,
      name,
      phone,
      email: email || null,
      loyaltyPoints: null
    });
    
    customerModal.classList.add("hidden");
    await loadCustomers();
    renderCustomersList();
  } catch (err) {
    alert("Error registering customer: " + err);
  }
}

async function openCustomerHistory(customer) {
  try {
    custHistoryName.textContent = `${customer.name} (${customer.phone})`;
    custHistoryList.innerHTML = "";
    
    const orders = await invoke("get_customer_orders", { customerId: customer.id });
    if (orders.length === 0) {
      custHistoryList.innerHTML = `<span style="text-align:center; color:var(--text-muted); padding:20px;">No transaction history found</span>`;
    } else {
      orders.forEach(o => {
        const item = document.createElement("div");
        item.className = "category-editor-item";
        item.style.padding = "10px";
        item.innerHTML = `
          <div style="text-align:left;">
            <div style="font-weight:600;">Bill ID: #${o.id}</div>
            <div style="font-size:0.75rem; color:var(--text-secondary); margin-top:2px;">Amount: ₹${o.total.toFixed(2)} | Date: ${formatIndianDate(o.created_at)}</div>
          </div>
          <button class="quick-role-btn" style="background:var(--primary); color:white; border:none; padding:2px 6px;">View Receipt</button>
        `;
        item.querySelector("button").addEventListener("click", () => reprintOrder(o.id));
        custHistoryList.appendChild(item);
      });
    }

    customerHistoryModal.classList.remove("hidden");
  } catch (err) {
    console.error(err);
  }
}

// 7. Reports & Past Completed Bills reprint
async function generateReport() {
  const from = convertIndianToIso(reportFromDate.value);
  const to = convertIndianToIso(reportToDate.value);
  if (!from || !to) return;

  try {
    const r = await invoke("get_sales_report", { startDate: from, endDate: to });
    reportTotalRevenue.textContent = `₹${r.total_sales.toFixed(2)}`;
    reportTotalTax.textContent = `₹${r.total_tax.toFixed(2)}`;
    reportOrderCount.textContent = r.order_count;
    reportAvgBill.textContent = `₹${r.average_ticket.toFixed(2)}`;

    // Get Payment summaries
    const pay = await invoke("get_payment_mode_summary", { startDate: from, endDate: to });
    reportCashSales.textContent = `₹${pay.cash_sales.toFixed(2)}`;
    reportUpiSales.textContent = `₹${pay.upi_sales.toFixed(2)}`;
    reportCardSales.textContent = `₹${pay.card_sales.toFixed(2)}`;
    reportMixedSales.textContent = `₹${pay.mixed_sales.toFixed(2)}`;

    // Top Selling Products report
    const productsReport = await invoke("get_product_sales_report", { startDate: from, endDate: to });
    reportProductsBody.innerHTML = "";
    if (productsReport.length === 0) {
      reportProductsBody.innerHTML = `<tr><td colspan="4" style="text-align:center; color:var(--text-muted); padding:20px;">No sales recorded in date range</td></tr>`;
    } else {
      productsReport.forEach(p => {
        const tr = document.createElement("tr");
        tr.innerHTML = `
          <td style="padding:6px;">${p.name}</td>
          <td style="padding:6px;">${p.category_name}</td>
          <td style="padding:6px; text-align:center;">${p.quantity}</td>
          <td style="padding:6px; text-align:right;">₹${p.total_sales.toFixed(2)}</td>
        `;
        reportProductsBody.appendChild(tr);
      });
    }

    // Past completed orders reprint history
    const pastOrders = await invoke("get_completed_orders");
    reportCompletedBillsList.innerHTML = "";
    if (pastOrders.length === 0) {
      reportCompletedBillsList.innerHTML = `<span style="text-align:center; color:var(--text-muted); padding:20px;">No completed transactions found</span>`;
    } else {
      pastOrders.forEach(o => {
        const card = document.createElement("div");
        card.className = "category-editor-item";
        card.style.padding = "10px";
        card.innerHTML = `
          <div style="text-align:left;">
            <div style="font-weight:600; color:var(--primary);">Bill ID: #${o.id}</div>
            <div style="font-size:0.75rem; color:var(--text-secondary); margin-top:2px;">Amount: ₹${o.total.toFixed(2)} | Date: ${formatIndianDate(o.created_at)}</div>
          </div>
          <button class="quick-role-btn" style="background:var(--primary); color:white; border:none; padding:4px 8px;">Receipt</button>
        `;
        card.querySelector("button").addEventListener("click", () => reprintOrder(o.id));
        reportCompletedBillsList.appendChild(card);
      });
    }
  } catch (err) {
    alert("Error generating report: " + err);
  }
}

async function reprintOrder(orderId) {
  try {
    const orderData = await invoke("get_order", { orderId });
    
    // Simulate opening checkout modal in read-only print mode
    checkoutModalAmount.textContent = `₹${orderData.header.total.toFixed(2)}`;
    checkoutPaymentMode.value = orderData.header.payment_mode || "Cash";
    checkoutSplitBlock.classList.add("hidden");
    checkoutChangeBlock.classList.add("hidden");
    checkoutChangeRow.classList.add("hidden");
    
    // Disables Finalize to prevent double billing records
    checkoutModalConfirm.disabled = true;

    // Populate Virtual Receipt
    receiptStoreName.textContent = restaurantInfo.name.toUpperCase();
    receiptStoreAddress.textContent = restaurantInfo.address || "";
    receiptStorePhone.textContent = restaurantInfo.phone ? `Ph: ${restaurantInfo.phone}` : "";
    receiptStoreGstin.textContent = restaurantInfo.gstin ? `GSTIN: ${restaurantInfo.gstin}` : "";
    receiptBillNumber.textContent = `#${orderData.header.id} (DUPLICATE)`;
    receiptDate.textContent = new Date(orderData.header.created_at).toLocaleString();
    receiptTable.textContent = orderData.header.table_name || "No Table";
    receiptCashier.textContent = "Cashier";

    if (orderData.header.customer_name) {
      receiptCustomerRow.classList.remove("hidden");
      receiptCustomer.textContent = `${orderData.header.customer_name}`;
    } else {
      receiptCustomerRow.classList.add("hidden");
    }

    // Populate Items
    receiptItemsBody.innerHTML = "";
    orderData.items.forEach(item => {
      const tr = document.createElement("tr");
      tr.innerHTML = `
        <td style="padding: 4px 0;">
          ${item.name}
          ${item.notes ? `<div style="font-size:0.6rem; color:#777; font-style:italic;">* ${item.notes}</div>` : ""}
        </td>
        <td style="text-align: center; padding: 4px 0;">${item.quantity}</td>
        <td style="text-align: right; padding: 4px 0;">₹${(item.price * item.quantity).toFixed(2)}</td>
      `;
      receiptItemsBody.appendChild(tr);
    });

    const discountAmount = orderData.header.subtotal * (orderData.header.discount / 100);
    const serviceAmount = orderData.header.subtotal * (orderData.header.service_charge / 100);

    receiptSubtotal.textContent = `₹${orderData.header.subtotal.toFixed(2)}`;
    receiptDiscount.textContent = `-₹${discountAmount.toFixed(2)}`;
    receiptService.textContent = `₹${serviceAmount.toFixed(2)}`;
    receiptTax.textContent = `₹${orderData.header.tax.toFixed(2)}`;
    receiptTotal.textContent = `₹${orderData.header.total.toFixed(2)}`;
    receiptFooterMsg.textContent = restaurantInfo.receipt_footer || "Thank you for dining with us!";

    // Hide history modals if open
    customerHistoryModal.classList.add("hidden");

    checkoutModal.classList.remove("hidden");
  } catch (err) {
    alert("Error fetching reprint details: " + err);
  }
}

// 8. Settings Config
function loadSettingsForm() {
  if (!restaurantInfo) return;
  settingsInputName.value = restaurantInfo.name;
  settingsInputGstin.value = restaurantInfo.gstin || "";
  settingsInputAddress.value = restaurantInfo.address || "";
  settingsInputPhone.value = restaurantInfo.phone || "";
  settingsInputEmail.value = restaurantInfo.email || "";
  settingsInputFooter.value = restaurantInfo.receipt_footer || "";
}

async function saveSettings(e) {
  e.preventDefault();
  const name = settingsInputName.value.trim();
  const gstin = settingsInputGstin.value.trim();
  const address = settingsInputAddress.value.trim();
  const phone = settingsInputPhone.value.trim();
  const email = settingsInputEmail.value.trim();
  const footer = settingsInputFooter.value.trim();

  if (!name) return;

  try {
    await invoke("update_restaurant_info", {
      name,
      logo: "",
      gstin: gstin || null,
      address: address || null,
      phone: phone || null,
      email: email || null,
      receiptFooter: footer || null
    });

    restaurantInfo = await invoke("get_restaurant_info");
    restaurantNameHeader.textContent = restaurantInfo.name;

    modalTitle.textContent = "Configuration Updated";
    modalMsg.textContent = "Restaurant information saved and loaded successfully.";
    successModal.classList.remove("hidden");
  } catch (err) {
    alert("Error saving settings configuration: " + err);
  }
}

// 9. Send KOT & Print KOT Features
async function sendKot() {
  if (cart.length === 0) return;

  // Block KOT if there are no new (un-KOT'd) items to send
  const hasNewItems = cart.some(item => !item.kot_id);
  if (!hasNewItems) {
    alert("No new items to send. All items have already been sent to the kitchen.");
    return;
  }

  const tableIdVal = cartTableSelect.value ? parseInt(cartTableSelect.value) : null;
  const custIdVal = cartCustomerSelect.value ? parseInt(cartCustomerSelect.value) : null;

  const orderItemsInput = cart.map(item => ({
    id: item.id || null,
    product_id: item.product.id,
    name: item.product.name,
    quantity: item.quantity,
    price: item.product.price,
    gst_rate: item.product.gst_rate,
    notes: item.notes || null
  }));

  try {
    let orderId = activeOrderId;
    if (activeOrderId) {
      await invoke("update_order", {
        orderId: activeOrderId,
        tableId: tableIdVal,
        customerId: custIdVal,
        notes: "",
        items: orderItemsInput,
        createdAt: new Date().toISOString(),
        username: currentUser.username
      });
    } else {
      orderId = await invoke("create_order", {
        tableId: tableIdVal,
        customerId: custIdVal,
        notes: "",
        items: orderItemsInput,
        createdAt: new Date().toISOString(),
        username: currentUser.username
      });
      activeOrderId = orderId;
    }

    activeOrderStatus = "Pending";

    // Print the generated KOT
    const kots = await invoke("get_kots_for_order", { orderId: orderId || activeOrderId });
    if (kots && kots.length > 0) {
      const pendingKots = kots.filter(k => k.print_count === 0);
      if (pendingKots.length > 0) {
        await printKot(pendingKots[pendingKots.length - 1]);
      } else {
        await printKot(kots[kots.length - 1]);
      }
    }

    // Reset cart UI
    const sentOrderId = orderId || activeOrderId;
    cart = [];
    activeOrderId = null;
    activeOrderStatus = null;
    cartTableSelect.value = "";
    cartCustomerSelect.value = "";
    renderCart();
    updateSelectColors();

    modalTitle.textContent = "KOT Sent to Kitchen";
    modalMsg.textContent = `Order #${sentOrderId} saved as Pending and sent to kitchen successfully.`;
    successModal.classList.remove("hidden");

    loadKots();
    renderTables();
  } catch (err) {
    alert("Error sending KOT: " + err);
  }
}

async function printKot(kot) {
  try {
    const newCount = await invoke("increment_kot_print_count", { kotId: kot.id, username: currentUser.username });
    kot.print_count = newCount;
  } catch (err) {
    console.error("Error updating print count in backend: ", err);
  }

  // Populate print layout
  document.getElementById("kot-print-title").textContent = `KOT #${kot.id}`;
  document.getElementById("kot-print-copy").textContent = `Print Copy: #${kot.print_count}`;
  document.getElementById("kot-print-table").textContent = `Table: ${kot.table_name || 'No Table'}`;
  
  const dateObj = new Date(kot.created_at);
  document.getElementById("kot-print-date").textContent = `Date/Time: ${dateObj.toLocaleDateString()} ${dateObj.toLocaleTimeString([], { hour: '2-digit', minute: '2-digit' })}`;

  const itemsBody = document.getElementById("kot-print-items-body");
  itemsBody.innerHTML = "";
  
  kot.items.forEach(it => {
    const isCancelled = it.quantity < 0;
    const qty = Math.abs(it.quantity);
    const name = isCancelled ? `${it.product_name} (CANCELLED)` : it.product_name;
    
    const tr = document.createElement("tr");
    tr.style.borderBottom = "1px dashed #eee";
    tr.innerHTML = `
      <td style="padding: 6px 0; font-weight: ${isCancelled ? 'normal' : 'bold'}; text-decoration: ${isCancelled ? 'line-through' : 'none'}; color: ${isCancelled ? '#777' : 'black'};">
        ${name}
        ${it.notes ? `<div style="font-size:0.75rem; font-weight:normal; font-style:italic; color:#555; margin-top:2px;">* Notes: ${it.notes}</div>` : ""}
      </td>
      <td style="text-align: center; padding: 6px 0; font-weight: bold; color: ${isCancelled ? '#777' : 'black'};">${qty}</td>
    `;
    itemsBody.appendChild(tr);
  });

  // Switch print mode
  document.body.classList.remove("print-receipt-mode");
  document.body.classList.add("print-kot-mode");

  // Trigger system print dialog
  window.print();

  // Reset print mode
  document.body.classList.remove("print-kot-mode");
  document.body.classList.add("print-receipt-mode");
  
  // Reload KOT displays
  loadKots();
}

async function printKotById(kotId) {
  try {
    const kot = await invoke("get_kot_by_id", { kotId });
    await printKot(kot);
  } catch (err) {
    alert("Error fetching KOT for printing: " + err);
  }
}

function openCancellationModal(target) {
  activeCancellationTarget = target;
  
  // Reset form
  cancelReasonSelect.value = "Customer requested cancellation";
  cancelReasonText.value = "";
  cancelReasonTextGroup.classList.add("hidden");
  cancelReasonText.required = false;
  
  // Authorization visibility (none required, single admin has full permissions)
  
  if (target.type === 'item') {
    const item = cart[target.itemIndex];
    cancelModalTitle.textContent = "Cancel Item";
    cancelModalItemName.textContent = `${item.product.name} (KOT #${item.kot_id})`;
    cancelQtyGroup.classList.remove("hidden");
    
    const maxQty = item.quantity - (item.cancelled_quantity || 0);
    cancelQtyInput.value = maxQty;
    cancelQtyInput.max = maxQty;
    cancelQtyInput.min = 1;
  } else {
    cancelModalTitle.textContent = "Cancel Entire Order";
    cancelModalItemName.textContent = `Order #${activeOrderId}`;
    cancelQtyGroup.classList.add("hidden");
  }
  
  cancelModal.classList.remove("hidden");
}

async function submitCancellation() {
  let reason = cancelReasonSelect.value;
  if (reason === "Other") {
    reason = cancelReasonText.value.trim();
    if (!reason) {
      alert("Please specify the cancellation reason.");
      return;
    }
  }
  
  let authorizer = currentUser.username;
  
  try {
    if (activeCancellationTarget.type === 'item') {
      const item = cart[activeCancellationTarget.itemIndex];
      const qty = parseInt(cancelQtyInput.value);
      const maxQty = item.quantity - (item.cancelled_quantity || 0);
      if (isNaN(qty) || qty < 1 || qty > maxQty) {
        alert(`Invalid cancellation quantity. Must be between 1 and ${maxQty}.`);
        return;
      }
      
      await invoke("cancel_order_item", {
        orderItemId: item.id,
        quantityToCancel: qty,
        cancelledBy: authorizer,
        reason: reason
      });
      
      cancelModal.classList.add("hidden");
      
      await resumeOrder(activeOrderId);
      loadKots();
      renderTables();
      
      modalTitle.textContent = "Item Cancelled";
      modalMsg.textContent = `${qty} x ${item.product.name} cancelled successfully and sent to kitchen monitor.`;
      successModal.classList.remove("hidden");
    } else {
      const cancelledOrderId = activeOrderId;
      await invoke("cancel_order", {
        orderId: activeOrderId,
        cancelledBy: authorizer,
        reason: reason
      });
      
      cancelModal.classList.add("hidden");
      
      cart = [];
      activeOrderId = null;
      activeOrderStatus = null;
      cartTableSelect.value = "";
      cartCustomerSelect.value = "";
      renderCart();
      updateSelectColors();
      
      loadKots();
      renderTables();
      
      modalTitle.textContent = "Order Cancelled";
      modalMsg.textContent = `Order #${cancelledOrderId} has been cancelled successfully.`;
      successModal.classList.remove("hidden");
    }
  } catch (err) {
    alert("Cancellation error: " + err);
  }
}
