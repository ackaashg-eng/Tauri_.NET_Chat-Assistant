-- =====================================================================
--  Lumen Home Electronics demo schema - Microsoft SQL Server 2019 or newer
--  (Express, Developer, Standard, Azure SQL all work).
--  Safe to run more than once: every object is created only if missing.
--  "GO" separates batches (the server splits the script on GO lines).
-- =====================================================================

IF OBJECT_ID(N'dbo.products', N'U') IS NULL
CREATE TABLE dbo.products (
    id               INT             NOT NULL PRIMARY KEY,
    sku              NVARCHAR(32)    NOT NULL UNIQUE,
    name             NVARCHAR(200)   NOT NULL,
    category         NVARCHAR(100)   NOT NULL,
    description      NVARCHAR(MAX)   NOT NULL,
    price            DECIMAL(10, 2)  NOT NULL CHECK (price >= 0),
    stock            INT             NOT NULL CONSTRAINT df_products_stock DEFAULT 0 CHECK (stock >= 0),
    warranty_months  INT             NOT NULL CONSTRAINT df_products_warranty DEFAULT 12,
    created_at       DATETIME2(0)    NOT NULL CONSTRAINT df_products_created DEFAULT SYSUTCDATETIME()
);
GO

IF OBJECT_ID(N'dbo.articles', N'U') IS NULL
CREATE TABLE dbo.articles (
    id          INT            NOT NULL PRIMARY KEY,
    title       NVARCHAR(300)  NOT NULL,
    category    NVARCHAR(100)  NOT NULL,
    body        NVARCHAR(MAX)  NOT NULL,
    tags        NVARCHAR(500)  NOT NULL CONSTRAINT df_articles_tags DEFAULT N'',
    updated_at  DATETIME2(0)   NOT NULL CONSTRAINT df_articles_updated DEFAULT SYSUTCDATETIME()
);
GO

IF NOT EXISTS (SELECT 1 FROM sys.indexes WHERE name = N'ix_products_category' AND object_id = OBJECT_ID(N'dbo.products'))
    CREATE INDEX ix_products_category ON dbo.products(category);
GO

IF NOT EXISTS (SELECT 1 FROM sys.indexes WHERE name = N'ix_articles_category' AND object_id = OBJECT_ID(N'dbo.articles'))
    CREATE INDEX ix_articles_category ON dbo.articles(category);
GO
