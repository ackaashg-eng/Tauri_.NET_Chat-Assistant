-- =====================================================================
--  ERP consultation tables - Microsoft SQL Server 2019 or newer.
--  A (fictional) ERP consultancy: clients book consultations with
--  consultants; consultations have meetings, orders and feedback.
--  Safe to run more than once: objects are created only if missing,
--  views are (re)created with CREATE OR ALTER.
-- =====================================================================

IF OBJECT_ID(N'dbo.erp_clients', N'U') IS NULL
CREATE TABLE dbo.erp_clients (
    id             INT IDENTITY(1,1) NOT NULL PRIMARY KEY,
    name           NVARCHAR(200)  NOT NULL CONSTRAINT uq_erp_clients_name UNIQUE,
    industry       NVARCHAR(100)  NOT NULL,
    country        NVARCHAR(100)  NOT NULL,
    erp_system     NVARCHAR(100)  NOT NULL,
    contact_name   NVARCHAR(150)  NOT NULL,
    contact_email  NVARCHAR(200)  NOT NULL,
    created_at     DATETIME2(0)   NOT NULL CONSTRAINT df_erp_clients_created DEFAULT SYSDATETIME(),
    created_by     NVARCHAR(100)  NULL
);
GO

IF OBJECT_ID(N'dbo.erp_consultants', N'U') IS NULL
CREATE TABLE dbo.erp_consultants (
    id           INT IDENTITY(1,1) NOT NULL PRIMARY KEY,
    name         NVARCHAR(150)  NOT NULL CONSTRAINT uq_erp_consultants_name UNIQUE,
    email        NVARCHAR(200)  NOT NULL,
    speciality   NVARCHAR(100)  NOT NULL,
    hourly_rate  DECIMAL(10, 2) NOT NULL CHECK (hourly_rate >= 0),
    active       BIT            NOT NULL CONSTRAINT df_erp_consultants_active DEFAULT 1
);
GO

IF OBJECT_ID(N'dbo.erp_consultations', N'U') IS NULL
CREATE TABLE dbo.erp_consultations (
    id                 INT IDENTITY(1,1) NOT NULL PRIMARY KEY,
    client_id          INT            NOT NULL REFERENCES dbo.erp_clients(id),
    consultant_id      INT            NULL     REFERENCES dbo.erp_consultants(id),
    title              NVARCHAR(200)  NOT NULL,
    erp_module         NVARCHAR(50)   NOT NULL,
    consultation_type  NVARCHAR(50)   NOT NULL,
    status             NVARCHAR(20)   NOT NULL CONSTRAINT df_erp_consultations_status DEFAULT N'Requested'
                       CONSTRAINT ck_erp_consultations_status CHECK (status IN (N'Requested', N'Scheduled', N'In progress', N'Completed', N'Cancelled')),
    priority           NVARCHAR(10)   NOT NULL CONSTRAINT df_erp_consultations_priority DEFAULT N'Normal'
                       CONSTRAINT ck_erp_consultations_priority CHECK (priority IN (N'Low', N'Normal', N'High', N'Urgent')),
    requested_at       DATETIME2(0)   NOT NULL CONSTRAINT df_erp_consultations_requested DEFAULT SYSDATETIME(),
    scheduled_start    DATETIME2(0)   NULL,
    duration_hours     DECIMAL(6, 1)  NULL CHECK (duration_hours > 0),
    description        NVARCHAR(MAX)  NOT NULL,
    outcome            NVARCHAR(MAX)  NULL,
    created_by         NVARCHAR(100)  NULL,
    updated_at         DATETIME2(0)   NULL,
    updated_by         NVARCHAR(100)  NULL
);
GO

IF OBJECT_ID(N'dbo.erp_meetings', N'U') IS NULL
CREATE TABLE dbo.erp_meetings (
    id                INT IDENTITY(1,1) NOT NULL PRIMARY KEY,
    consultation_id   INT            NOT NULL REFERENCES dbo.erp_consultations(id),
    meeting_at        DATETIME2(0)   NOT NULL,
    duration_minutes  INT            NOT NULL CHECK (duration_minutes BETWEEN 5 AND 600),
    mode              NVARCHAR(20)   NOT NULL CONSTRAINT ck_erp_meetings_mode CHECK (mode IN (N'Online', N'On-site', N'Phone')),
    attendees         NVARCHAR(500)  NOT NULL CONSTRAINT df_erp_meetings_attendees DEFAULT N'',
    agenda            NVARCHAR(MAX)  NOT NULL,
    notes             NVARCHAR(MAX)  NULL,
    action_items      NVARCHAR(MAX)  NULL,
    created_at        DATETIME2(0)   NOT NULL CONSTRAINT df_erp_meetings_created DEFAULT SYSDATETIME(),
    created_by        NVARCHAR(100)  NULL
);
GO

IF OBJECT_ID(N'dbo.erp_orders', N'U') IS NULL
CREATE TABLE dbo.erp_orders (
    id               INT IDENTITY(1,1) NOT NULL PRIMARY KEY,
    -- ORD-00001, ORD-00002 ... generated from the id
    order_number     AS (N'ORD-' + RIGHT(N'00000' + CAST(id AS NVARCHAR(10)), 5)) PERSISTED,
    client_id        INT            NOT NULL REFERENCES dbo.erp_clients(id),
    consultation_id  INT            NULL     REFERENCES dbo.erp_consultations(id),
    order_date       DATE           NOT NULL CONSTRAINT df_erp_orders_date DEFAULT CAST(SYSDATETIME() AS DATE),
    service          NVARCHAR(200)  NOT NULL,
    hours            DECIMAL(7, 1)  NOT NULL CHECK (hours >= 0),
    amount           DECIMAL(14, 2) NOT NULL CHECK (amount >= 0),
    currency         CHAR(3)        NOT NULL CONSTRAINT df_erp_orders_currency DEFAULT 'USD',
    status           NVARCHAR(20)   NOT NULL CONSTRAINT df_erp_orders_status DEFAULT N'Quoted'
                     CONSTRAINT ck_erp_orders_status CHECK (status IN (N'Quoted', N'Confirmed', N'Invoiced', N'Paid', N'Cancelled')),
    notes            NVARCHAR(MAX)  NULL,
    created_at       DATETIME2(0)   NOT NULL CONSTRAINT df_erp_orders_created DEFAULT SYSDATETIME(),
    created_by       NVARCHAR(100)  NULL,
    updated_at       DATETIME2(0)   NULL,
    updated_by       NVARCHAR(100)  NULL
);
GO

IF OBJECT_ID(N'dbo.erp_feedback', N'U') IS NULL
CREATE TABLE dbo.erp_feedback (
    id                  INT IDENTITY(1,1) NOT NULL PRIMARY KEY,
    consultation_id     INT            NOT NULL REFERENCES dbo.erp_consultations(id),
    submitted_at        DATETIME2(0)   NOT NULL CONSTRAINT df_erp_feedback_submitted DEFAULT SYSDATETIME(),
    rating              TINYINT        NOT NULL CHECK (rating BETWEEN 1 AND 5),
    comments            NVARCHAR(MAX)  NOT NULL,
    would_recommend     BIT            NOT NULL,
    follow_up_required  BIT            NOT NULL CONSTRAINT df_erp_feedback_follow DEFAULT 0,
    created_by          NVARCHAR(100)  NULL
);
GO

IF NOT EXISTS (SELECT 1 FROM sys.indexes WHERE name = N'ix_erp_consultations_client' AND object_id = OBJECT_ID(N'dbo.erp_consultations'))
    CREATE INDEX ix_erp_consultations_client ON dbo.erp_consultations(client_id, status);
GO
IF NOT EXISTS (SELECT 1 FROM sys.indexes WHERE name = N'ix_erp_consultations_consultant' AND object_id = OBJECT_ID(N'dbo.erp_consultations'))
    CREATE INDEX ix_erp_consultations_consultant ON dbo.erp_consultations(consultant_id, status);
GO
IF NOT EXISTS (SELECT 1 FROM sys.indexes WHERE name = N'ix_erp_meetings_consultation' AND object_id = OBJECT_ID(N'dbo.erp_meetings'))
    CREATE INDEX ix_erp_meetings_consultation ON dbo.erp_meetings(consultation_id, meeting_at);
GO
IF NOT EXISTS (SELECT 1 FROM sys.indexes WHERE name = N'ix_erp_orders_client' AND object_id = OBJECT_ID(N'dbo.erp_orders'))
    CREATE INDEX ix_erp_orders_client ON dbo.erp_orders(client_id, status);
GO
IF NOT EXISTS (SELECT 1 FROM sys.indexes WHERE name = N'ix_erp_feedback_consultation' AND object_id = OBJECT_ID(N'dbo.erp_feedback'))
    CREATE INDEX ix_erp_feedback_consultation ON dbo.erp_feedback(consultation_id);
GO

-- ---------------------------------------------------------------------
--  Search views: one row per searchable thing, with client and consultant
--  NAMES joined in, so a search for a client name also finds its rows.
--  These are what the "Sources" in appsettings.json point at.
-- ---------------------------------------------------------------------
CREATE OR ALTER VIEW dbo.v_erp_consultations AS
SELECT c.id, c.title, cl.name AS client, COALESCE(co.name, N'(unassigned)') AS consultant,
       c.erp_module, c.consultation_type, c.status, c.priority,
       c.requested_at, c.scheduled_start, c.duration_hours, c.description, c.outcome
FROM dbo.erp_consultations c
JOIN dbo.erp_clients cl ON cl.id = c.client_id
LEFT JOIN dbo.erp_consultants co ON co.id = c.consultant_id;
GO

CREATE OR ALTER VIEW dbo.v_erp_meetings AS
SELECT m.id, N'Meeting: ' + c.title AS title, cl.name AS client, COALESCE(co.name, N'(unassigned)') AS consultant,
       m.consultation_id, m.meeting_at, m.duration_minutes, m.mode, m.attendees, m.agenda, m.notes, m.action_items
FROM dbo.erp_meetings m
JOIN dbo.erp_consultations c ON c.id = m.consultation_id
JOIN dbo.erp_clients cl ON cl.id = c.client_id
LEFT JOIN dbo.erp_consultants co ON co.id = c.consultant_id;
GO

CREATE OR ALTER VIEW dbo.v_erp_feedback AS
SELECT f.id, N'Feedback: ' + c.title AS title, cl.name AS client, COALESCE(co.name, N'(unassigned)') AS consultant,
       f.consultation_id, c.erp_module, f.rating, f.would_recommend, f.follow_up_required, f.submitted_at, f.comments
FROM dbo.erp_feedback f
JOIN dbo.erp_consultations c ON c.id = f.consultation_id
JOIN dbo.erp_clients cl ON cl.id = c.client_id
LEFT JOIN dbo.erp_consultants co ON co.id = c.consultant_id;
GO

CREATE OR ALTER VIEW dbo.v_erp_orders AS
SELECT o.id, o.order_number + N' ' + o.service AS title, o.order_number, cl.name AS client, o.consultation_id,
       o.order_date, o.service, o.hours, o.amount, o.currency, o.status, o.notes
FROM dbo.erp_orders o
JOIN dbo.erp_clients cl ON cl.id = o.client_id;
GO
