-- Add soft delete support to products table
ALTER TABLE products ADD COLUMN deleted_at TIMESTAMPTZ DEFAULT NULL;

-- Index for filtering out deleted products efficiently
CREATE INDEX idx_products_deleted_at ON products(deleted_at)
    WHERE deleted_at IS NULL;
