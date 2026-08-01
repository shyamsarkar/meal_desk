# MealDesk

## Overview

MealDesk is a desktop restaurant billing (POS) application built with
**Rust + Tauri**.

### Goals

-   Windows-first desktop application
-   Cross-platform (Windows, Linux)
-   Offline-first
-   SQLite database
-   Fast and lightweight
-   Clean, responsive UI

## Technology Stack

-   Rust
-   Tauri
-   SQLite

## MVP Features

### Authentication

-   Login (admin/admin123)

### Restaurant

-   Name
-   Logo
-   GSTIN
-   Address
-   Phone
-   Email
-   Receipt footer

### Menu

-   Categories
-   Products
-   Price
-   GST rate
-   Image (optional)
-   Availability

### Billing

-   New bill
-   Add/remove items
-   Product search
-   Quantity update
-   Item notes
-   Discounts
-   GST calculation
-   Service charge
-   Round-off
-   Hold/Resume bill
-   Cancel bill

### Payments

-   Cash
-   UPI
-   Card
-   Mixed payment
-   Split payment
-   Change calculation

### Receipts

-   Thermal printing
-   PDF receipt
-   Reprint
-   Duplicate copy
-   QR code
-   Custom footer

### Table Management

-   Table status
-   Merge tables
-   Transfer tables

### Kitchen

-   Kitchen Order Tickets (KOT)
-   Pending / Preparing / Ready / Completed

### Inventory

-   Stock
-   Purchases
-   Adjustments
-   Low-stock alerts

### Customers

-   Walk-in customer
-   Customer database
-   Loyalty points
-   Order history

### Reports

-   Daily sales
-   Monthly sales
-   GST report
-   Product & category sales
-   Payment summary
-   Cash drawer report

### Settings

-   GST
-   Theme
-   Printer
-   Backup / Restore

### Hardware

-   Thermal printer
-   Cash drawer

### Future

-   Multi-branch
-   Cloud sync
-   Mobile app
-   Online ordering

## Proposed Project Structure

``` text
mealdesk/
├── src-tauri/
├── src/
├── core/
│   ├── billing/
│   ├── inventory/
│   ├── reports/
│   ├── printer/
│   ├── auth/
│   └── customers/
├── database/
└── shared/
```

## Development Plan

1.  Finalize requirements
2.  Design database schema
3.  Design UI screens
4.  Build authentication
5.  Build billing module
6.  Integrate printing
7.  Add reports
8.  Package for Windows
