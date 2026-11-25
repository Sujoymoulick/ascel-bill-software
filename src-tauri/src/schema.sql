CREATE TABLE business (
    id INTEGER PRIMARY KEY CHECK (id = 1),
    data TEXT NOT NULL CHECK (json_valid(data))
) STRICT;
CREATE TABLE customers (
    id TEXT PRIMARY KEY,
    name TEXT NOT NULL,
    data TEXT NOT NULL CHECK (json_valid(data))
) STRICT;
CREATE INDEX customer_name ON customers(name COLLATE NOCASE);
CREATE TABLE items (
    id TEXT PRIMARY KEY,
    name TEXT NOT NULL,
    hsn TEXT NOT NULL,
    stock INTEGER CHECK (stock >= 0),
    data TEXT NOT NULL CHECK (json_valid(data))
) STRICT;
CREATE INDEX item_name ON items(name COLLATE NOCASE);
CREATE TABLE drafts (
    id TEXT PRIMARY KEY,
    updated TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
    data TEXT NOT NULL CHECK (json_valid(data))
) STRICT;
CREATE TABLE sequences (
    fiscal_year TEXT PRIMARY KEY,
    next INTEGER NOT NULL CHECK (next > 0)
) STRICT;
CREATE TABLE invoices (
    id TEXT PRIMARY KEY,
    number TEXT NOT NULL UNIQUE,
    date TEXT NOT NULL,
    customer TEXT NOT NULL,
    payment TEXT NOT NULL,
    total INTEGER NOT NULL,
    tax INTEGER NOT NULL,
    data TEXT NOT NULL CHECK (json_valid(data))
) STRICT;
CREATE INDEX invoice_date ON invoices(date DESC);
CREATE TABLE stock_movements (
    invoice_id TEXT NOT NULL REFERENCES invoices(id),
    item_id TEXT NOT NULL REFERENCES items(id),
    quantity INTEGER NOT NULL CHECK (quantity > 0),
    PRIMARY KEY(invoice_id, item_id)
) STRICT;
CREATE TABLE audit (
    id INTEGER PRIMARY KEY,
    recorded TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
    action TEXT NOT NULL,
    reference TEXT NOT NULL
) STRICT;
