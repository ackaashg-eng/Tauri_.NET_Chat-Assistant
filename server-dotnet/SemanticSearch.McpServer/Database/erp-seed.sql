-- =====================================================================
--  Sample data for the (fictional) ERP consultancy - Microsoft SQL Server.
--  Generated file. Only inserted when dbo.erp_consultations is empty.
--  All companies, people and e-mail addresses are made up.
-- =====================================================================

SET IDENTITY_INSERT dbo.erp_clients ON;
INSERT INTO dbo.erp_clients (id, name, industry, country, erp_system, contact_name, contact_email, created_by) VALUES
(1, N'Lanka Spice Exporters (Pvt) Ltd', N'Food and agriculture', N'Sri Lanka', N'SAP Business One', N'Dilani Perera', N'dilani@lankaspiceexporters.example', N'seed'),
(2, N'Harbour Freight Logistics', N'Logistics', N'Sri Lanka', N'Microsoft Dynamics 365 Business Central', N'Ravi Fernando', N'ravi@harbourfreightlogistics.example', N'seed'),
(3, N'Islandwear Garments', N'Apparel manufacturing', N'Sri Lanka', N'Odoo', N'Shanika Silva', N'shanika@islandweargarments.example', N'seed'),
(4, N'Maldive Breeze Resorts', N'Hospitality', N'Maldives', N'Oracle NetSuite', N'Ahmed Shareef', N'ahmed@maldivebreezeresorts.example', N'seed'),
(5, N'Kandy Hills Tea Estates', N'Agriculture', N'Sri Lanka', N'ERPNext', N'Nuwan Jayasinghe', N'nuwan@kandyhillsteaestates.example', N'seed'),
(6, N'Bluepeak Pharmaceuticals', N'Healthcare', N'India', N'SAP Business One', N'Priya Raman', N'priya@bluepeakpharmaceuticals.example', N'seed'),
(7, N'Northgate Engineering Ltd', N'Manufacturing', N'United Kingdom', N'Microsoft Dynamics 365 Business Central', N'Oliver Grant', N'oliver@northgateengineeringltd.example', N'seed'),
(8, N'Coral Coast Retail', N'Retail', N'Australia', N'Odoo', N'Mia Thompson', N'mia@coralcoastretail.example', N'seed'),
(9, N'Sunrise Solar Solutions', N'Energy', N'Sri Lanka', N'Odoo', N'Kasun Wickramasinghe', N'kasun@sunrisesolarsolutions.example', N'seed'),
(10, N'Meridian Healthcare Labs', N'Healthcare', N'Singapore', N'Oracle NetSuite', N'Wei Ling Tan', N'wei@meridianhealthcarelabs.example', N'seed'),
(11, N'Peak Furniture Makers', N'Manufacturing', N'Sri Lanka', N'ERPNext', N'Chamara Bandara', N'chamara@peakfurnituremakers.example', N'seed'),
(12, N'Orchid Hotels Group', N'Hospitality', N'Sri Lanka', N'SAP Business One', N'Anjali Gunawardena', N'anjali@orchidhotelsgroup.example', N'seed');
SET IDENTITY_INSERT dbo.erp_clients OFF;
GO

SET IDENTITY_INSERT dbo.erp_consultants ON;
INSERT INTO dbo.erp_consultants (id, name, email, speciality, hourly_rate, active) VALUES
(1, N'Tharindu Senanayake', N'tharindu@consultancy.example', N'Finance and accounting', 85, 1),
(2, N'Sarah Mitchell', N'sarah@consultancy.example', N'Supply chain and inventory', 95, 1),
(3, N'Imran Hassan', N'imran@consultancy.example', N'Manufacturing', 90, 1),
(4, N'Nadeesha Rathnayake', N'nadeesha@consultancy.example', N'HR and payroll', 75, 1),
(5, N'Arjun Mehta', N'arjun@consultancy.example', N'Integration and development', 110, 1),
(6, N'Emily Clarke', N'emily@consultancy.example', N'Training and change management', 70, 1),
(7, N'Ruwan Dissanayake', N'ruwan@consultancy.example', N'Data migration and reporting', 80, 1),
(8, N'Lakmini Abeysekara', N'lakmini@consultancy.example', N'CRM and sales', 80, 1),
(9, N'David Brooks', N'david@consultancy.example', N'Finance and accounting', 90, 0);
SET IDENTITY_INSERT dbo.erp_consultants OFF;
GO

SET IDENTITY_INSERT dbo.erp_consultations ON;
INSERT INTO dbo.erp_consultations (id, client_id, consultant_id, title, erp_module, consultation_type, status, priority, requested_at, scheduled_start, duration_hours, description, outcome, created_by) VALUES
(1, 1, 1, N'Month-end close takes too long', N'Finance', N'Support', N'Completed', N'Normal', '2026-02-03T09:15:00', '2026-02-10T09:30:00', 4, N'The finance team needs five days to close the month in SAP Business One. Journal entries are posted by hand and bank reconciliation is done in Excel.', N'Set up recurring journal templates and the bank statement import. Month-end close reduced from five days to two.', N'seed'),
(2, 1, 4, N'EPF and ETF payroll setup', N'HR & Payroll', N'Implementation', N'Completed', N'High', '2026-03-02T11:00:00', '2026-03-09T10:00:00', 12, N'Configure payroll for 140 employees, including EPF and ETF contributions, overtime and monthly payslips.', N'Payroll configured and run in parallel with the old system for one month. Payslips are now emailed automatically.', N'seed'),
(3, 2, 2, N'Warehouse bin locations and stock counts', N'Inventory', N'Implementation', N'Completed', N'Normal', '2026-02-15T14:20:00', '2026-02-24T09:00:00', 16, N'Introduce bin locations in two warehouses and set up cycle counting so stock levels on the system match the shelves.', N'Bin locations live in both warehouses. The first cycle count showed less than 1% variance.', N'seed'),
(4, 2, 5, N'Integrate courier tracking API', N'Integration', N'Customisation', N'In progress', N'High', '2026-09-20T10:30:00', '2026-10-05T10:00:00', 24, N'Connect the courier partner''s tracking API so shipment status updates sales orders in Business Central automatically.', NULL, N'seed'),
(5, 3, 3, N'Bills of materials for a new product line', N'Manufacturing', N'Implementation', N'Completed', N'Normal', '2026-04-01T08:45:00', '2026-04-14T09:00:00', 20, N'Set up multi-level bills of materials and work orders for a new denim range, including cutting and stitching operations.', N'Bills of materials and routings created for 18 styles. Production planners trained to maintain them.', N'seed'),
(6, 3, 6, N'Training for production supervisors', N'Manufacturing', N'Training', N'Completed', N'Low', '2026-05-05T10:00:00', '2026-05-12T13:00:00', 6, N'Hands-on workshop for supervisors on recording production output and scrap in Odoo.', N'Twelve supervisors trained. A quick reference guide was delivered in Sinhala and English.', N'seed'),
(7, 3, 3, N'Shop floor tablets slow to record output', N'Manufacturing', N'Support', N'In progress', N'Urgent', '2026-10-28T07:50:00', '2026-11-03T09:00:00', 8, N'Recording finished output on the shop floor tablets takes almost a minute per entry and the screen often freezes during peak shifts.', NULL, N'seed'),
(8, 4, 1, N'Multi-currency revenue reporting', N'Finance', N'Customisation', N'Completed', N'Normal', '2026-03-10T12:00:00', '2026-03-20T10:00:00', 10, N'Resort revenue arrives in USD, EUR and MVR. Management needs consolidated revenue reports in USD with correct exchange rates.', N'Exchange rate feed and consolidated revenue dashboard delivered.', N'seed'),
(9, 4, 7, N'Migrate supplier data from the legacy system', N'Purchasing', N'Data migration', N'Completed', N'Normal', '2026-01-12T09:00:00', '2026-01-26T09:00:00', 14, N'Move 2,300 supplier records and open purchase orders from the old accounting package into NetSuite.', N'Supplier data cleansed and imported. 46 duplicate suppliers were merged.', N'seed'),
(10, 4, 2, N'Food and beverage stock across islands', N'Inventory', N'Health check', N'Scheduled', N'Normal', '2026-10-30T15:10:00', '2026-11-18T09:00:00', 6, N'Review how kitchen stock is transferred between three island resorts. Stock levels on the system do not match physical counts.', NULL, N'seed'),
(11, 5, 7, N'Move tea estate records to ERPNext', N'Projects', N'Data migration', N'Completed', N'High', '2026-02-01T08:30:00', '2026-02-16T08:30:00', 30, N'Migrate field, plucking and worker records from spreadsheets into ERPNext before the new financial year.', N'Opening balances and estate master data imported. Cut-over completed on time.', N'seed'),
(12, 5, 4, N'Daily wage calculation for estate workers', N'HR & Payroll', N'Customisation', N'In progress', N'High', '2026-09-15T10:00:00', '2026-10-01T09:00:00', 18, N'Workers are paid by the kilograms of tea plucked each day. Standard payroll cannot calculate daily wages from plucking weights.', NULL, N'seed'),
(13, 6, 1, N'GST returns from SAP Business One', N'Finance', N'Support', N'Completed', N'High', '2026-04-20T16:40:00', '2026-04-22T10:00:00', 5, N'The GST report does not match the purchase register and the tax return is due next week.', N'Tax codes on 37 vendor invoices corrected. The GST return was filed on time.', N'seed'),
(14, 6, 2, N'Batch and expiry tracking for medicines', N'Inventory', N'Implementation', N'Completed', N'High', '2026-05-18T09:20:00', '2026-06-01T09:00:00', 22, N'Pharmacy stock must be tracked by batch number and expiry date, with first-expiry-first-out picking.', N'Batch tracking is live and expiry alerts are sent 90 days in advance.', N'seed'),
(15, 6, 6, N'Training new warehouse staff', N'Inventory', N'Training', N'Scheduled', N'Normal', '2026-10-25T11:00:00', '2026-11-20T10:00:00', 4, N'Six new warehouse staff need training on goods receipt, batch selection and picking.', NULL, N'seed'),
(16, 7, 3, N'MRP planning for engineering parts', N'Manufacturing', N'Health check', N'Completed', N'Normal', '2026-06-03T13:00:00', '2026-06-15T09:30:00', 8, N'Material requirements planning suggests far too many purchase orders and the planners no longer trust it.', N'Reorder points and lead times corrected. Suggested purchase orders fell by 60%.', N'seed'),
(17, 7, 5, N'Connect Business Central to the CAD system', N'Integration', N'Customisation', N'Requested', N'Normal', '2026-11-02T10:15:00', NULL, NULL, N'Engineering wants item and bill of materials changes in the CAD system to sync into Business Central automatically.', NULL, N'seed'),
(18, 7, 1, N'Upgrade to the latest Business Central version', N'Finance', N'Upgrade', N'Scheduled', N'High', '2026-10-15T09:00:00', '2026-11-24T08:00:00', 16, N'The current version reaches end of support in December. Upgrade, test the customisations and retrain finance users.', NULL, N'seed'),
(19, 8, 8, N'Customer loyalty programme in Odoo', N'CRM', N'Implementation', N'Completed', N'Normal', '2026-03-05T10:00:00', '2026-03-18T09:00:00', 14, N'Launch a points-based loyalty programme across 9 stores and the online shop.', N'Loyalty cards live in all stores. 4,800 members registered in the first month.', N'seed'),
(20, 8, 5, N'Shopify orders not syncing', N'Integration', N'Support', N'Completed', N'Urgent', '2026-07-08T08:05:00', '2026-07-08T14:00:00', 3, N'Online orders from Shopify stopped appearing in Odoo on Monday, so the warehouse is not shipping them.', N'Expired API token renewed and a monitoring alert added for the order sync.', N'seed'),
(21, 8, 7, N'Sales dashboards for store managers', N'Reporting', N'Customisation', N'In progress', N'Normal', '2026-10-10T12:30:00', '2026-10-20T10:00:00', 12, N'Store managers want daily sales, margin and stock-out dashboards on their phones.', NULL, N'seed'),
(22, 9, 8, N'Track solar installation leads', N'CRM', N'Implementation', N'Completed', N'Normal', '2026-04-08T09:40:00', '2026-04-20T10:00:00', 10, N'The sales team tracks rooftop solar enquiries in WhatsApp and Excel, and leads are getting lost.', N'CRM pipeline configured with site survey stages. WhatsApp enquiries now create leads automatically.', N'seed'),
(23, 9, 6, N'Project costing training', N'Projects', N'Training', N'Completed', N'Low', '2026-06-10T10:00:00', '2026-06-17T14:00:00', 4, N'Project managers need to learn how to record time and materials against installation projects.', N'Training delivered. Project managers now log their hours every week.', N'seed'),
(24, 9, 1, N'Invoices stuck in draft', N'Finance', N'Support', N'Requested', N'Urgent', '2026-11-06T08:20:00', NULL, NULL, N'Since yesterday invoices cannot be validated and stay in draft with an error about the fiscal position. Customers are waiting for their bills.', NULL, N'seed'),
(25, 10, 7, N'Lab test volume reporting', N'Reporting', N'Customisation', N'Completed', N'Normal', '2026-05-02T11:00:00', '2026-05-14T10:00:00', 9, N'Management wants monthly reports of test volumes, turnaround times and revenue for each lab.', N'Saved searches and a dashboard built. The report is emailed every Monday.', N'seed'),
(26, 10, 4, N'Shift allowances in payroll', N'HR & Payroll', N'Customisation', N'Cancelled', N'Normal', '2026-07-01T09:00:00', NULL, NULL, N'Add night shift and weekend allowances to payroll.', N'Cancelled by the client. Their outsourced payroll provider will handle allowances.', N'seed'),
(27, 11, 3, N'Made-to-order furniture production', N'Manufacturing', N'Implementation', N'In progress', N'Normal', '2026-09-01T10:00:00', '2026-09-15T09:00:00', 28, N'Every order is custom sized. Bills of materials and work orders must be created from the sales order configuration.', NULL, N'seed'),
(28, 11, 2, N'Timber stock valuation wrong', N'Inventory', N'Support', N'Completed', N'High', '2026-08-04T15:30:00', '2026-08-06T10:00:00', 5, N'The value of timber stock on the balance sheet is far higher than expected after the last purchase.', N'Unit of measure conversion on teak purchases fixed and the stock revalued.', N'seed'),
(29, 12, 1, N'Year-end close and audit support', N'Finance', N'Support', N'Scheduled', N'High', '2026-11-01T10:00:00', '2026-11-27T09:00:00', 12, N'Prepare for the year-end: fixed asset depreciation, accruals and audit schedules.', NULL, N'seed'),
(30, 12, 8, N'Banquet booking quotations', N'Sales', N'Customisation', N'Requested', N'Normal', '2026-11-04T13:45:00', NULL, NULL, N'The events team wants to create banquet quotations with menus and room hire in SAP Business One instead of Word documents.', NULL, N'seed'),
(31, 12, 6, N'Onboarding new accounts staff', N'Finance', N'Training', N'Completed', N'Low', '2026-08-20T09:00:00', '2026-09-02T09:30:00', 6, N'Five new accounts staff need training on accounts payable, supplier payments and petty cash.', N'Training completed and all staff passed the practical assessment.', N'seed'),
(32, 2, 4, N'Leave and attendance rules', N'HR & Payroll', N'Implementation', N'Completed', N'Normal', '2026-06-22T10:00:00', '2026-07-06T09:00:00', 10, N'Set up annual, casual and medical leave rules and import attendance from the fingerprint machines.', N'Leave policies configured. Fingerprint attendance now imports every day.', N'seed'),
(33, 5, 2, N'Fertiliser stock at estate stores', N'Inventory', N'Health check', N'Requested', N'Low', '2026-11-05T09:30:00', NULL, NULL, N'Estate managers cannot see how much fertiliser is left in each estate store.', NULL, N'seed'),
(34, 1, 5, N'Export documents from sales orders', N'Sales', N'Customisation', N'In progress', N'Normal', '2026-10-02T11:10:00', '2026-10-16T10:00:00', 15, N'Generate commercial invoices, packing lists and certificates of origin from sales orders for spice exports.', NULL, N'seed');
SET IDENTITY_INSERT dbo.erp_consultations OFF;
GO

SET IDENTITY_INSERT dbo.erp_meetings ON;
INSERT INTO dbo.erp_meetings (id, consultation_id, meeting_at, duration_minutes, mode, attendees, agenda, notes, action_items, created_by) VALUES
(1, 1, '2026-02-10T09:30:00', 120, N'On-site', N'Dilani Perera, finance team', N'Review the month-end close checklist', N'Most of the delay comes from typing bank statements into Excel and posting the same accruals every month.', N'Set up bank statement import; create recurring journal templates.', N'seed'),
(2, 2, '2026-03-09T10:00:00', 180, N'On-site', N'Dilani Perera, HR manager', N'Payroll design workshop', N'Agreed EPF 8% and 12%, ETF 3%, overtime at 1.5 times the hourly rate.', N'Load employee master data; prepare a parallel run for March.', N'seed'),
(3, 2, '2026-04-06T15:00:00', 60, N'Online', N'Dilani Perera', N'Parallel run results', N'Two employees had wrong overtime rates. Everything else matched the old payroll.', N'Fix overtime rates; switch off the old payroll in April.', N'seed'),
(4, 3, '2026-02-24T09:00:00', 240, N'On-site', N'Ravi Fernando, warehouse supervisors', N'Design bin locations for both warehouses', N'Agreed aisle-rack-shelf naming. Fast movers go near dispatch.', N'Print bin labels; schedule the first cycle count.', N'seed'),
(5, 4, '2026-10-05T10:00:00', 90, N'Online', N'Ravi Fernando, IT team', N'Courier API kick-off', N'The courier sandbox credentials only arrived on day three, which delayed development.', N'Build the status mapping; test with 50 sample shipments.', N'seed'),
(6, 4, '2026-10-26T10:00:00', 45, N'Online', N'Ravi Fernando', N'Progress check', N'Client unhappy about the delay. Webhook retries are still failing for some shipments.', N'Send weekly status updates; fix webhook retries before go-live.', N'seed'),
(7, 5, '2026-04-14T09:00:00', 240, N'On-site', N'Shanika Silva, production planners', N'Bills of materials workshop', N'Cutting, stitching and washing defined as separate operations.', N'Create BOMs for the first 6 styles; review together next week.', N'seed'),
(8, 6, '2026-05-12T13:00:00', 180, N'On-site', N'Production supervisors', N'Supervisor training session', N'Supervisors practised recording output and scrap on the tablets.', N'Share the quick reference guide in Sinhala and English.', N'seed'),
(9, 7, '2026-11-03T09:00:00', 120, N'On-site', N'Shanika Silva, shop floor supervisors', N'Investigate slow tablets', N'Each output entry takes 40 to 60 seconds. The Wi-Fi signal is weak at the far end of the factory floor.', N'Run a network test; check Odoo server load during peak shift.', N'seed'),
(10, 8, '2026-03-20T10:00:00', 90, N'Online', N'Ahmed Shareef, finance', N'Revenue reporting requirements', N'Revenue must be shown in USD, with daily exchange rates from the central bank.', N'Connect the exchange rate feed; build a consolidated dashboard.', N'seed'),
(11, 9, '2026-01-26T09:00:00', 60, N'Online', N'Ahmed Shareef, purchasing', N'Supplier data mapping', N'Found many duplicate suppliers with slightly different names.', N'Client to confirm which duplicates can be merged.', N'seed'),
(12, 10, '2026-11-18T09:00:00', 360, N'On-site', N'Resort managers, chefs, storekeepers', N'Stock transfer health check across three islands', NULL, NULL, N'seed'),
(13, 11, '2026-02-16T08:30:00', 240, N'On-site', N'Nuwan Jayasinghe, estate clerks', N'Cut-over day', N'Spreadsheets were messy, but all opening balances were loaded and checked.', N'Lock the old spreadsheets; start daily entry in ERPNext.', N'seed'),
(14, 12, '2026-10-01T09:00:00', 120, N'On-site', N'Nuwan Jayasinghe, estate payroll clerks', N'Plucking-based wage rules', N'Daily wage equals kilograms plucked times the rate, plus an attendance allowance.', N'Build the wage calculation; test with September data.', N'seed'),
(15, 12, '2026-10-29T14:00:00', 60, N'Online', N'Nuwan Jayasinghe', N'Test results review', N'Test calculations matched the manual wage sheets for 3 of 4 estates.', N'Fix the rounding difference at the fourth estate.', N'seed'),
(16, 13, '2026-04-22T10:00:00', 150, N'Online', N'Priya Raman, accountant', N'GST reconciliation', N'37 vendor invoices had the wrong tax code.', N'Correct the tax codes; rerun the GST report before filing.', N'seed'),
(17, 14, '2026-06-01T09:00:00', 240, N'On-site', N'Priya Raman, pharmacy and warehouse leads', N'Batch and expiry design', N'Picking must suggest the batch with the earliest expiry date.', N'Enable batch management; load batch numbers for current stock.', N'seed'),
(18, 15, '2026-11-20T10:00:00', 240, N'On-site', N'Six new warehouse staff', N'Warehouse training: goods receipt and picking', NULL, NULL, N'seed'),
(19, 16, '2026-06-15T09:30:00', 180, N'On-site', N'Oliver Grant, planners', N'MRP health check', N'Lead times were years out of date and reorder points were too high.', N'Update lead times; rerun MRP and compare with last month.', N'seed'),
(20, 17, '2026-11-12T11:00:00', 45, N'Phone', N'Oliver Grant', N'Scoping call for the CAD integration', NULL, NULL, N'seed'),
(21, 18, '2026-11-24T08:00:00', 480, N'On-site', N'Oliver Grant, finance users', N'Business Central upgrade day', NULL, NULL, N'seed'),
(22, 19, '2026-03-18T09:00:00', 120, N'Online', N'Mia Thompson, store managers', N'Loyalty programme design', N'One point per dollar; rewards at 500 points.', N'Configure the loyalty programme; train store staff.', N'seed'),
(23, 20, '2026-07-08T14:00:00', 60, N'Online', N'Mia Thompson, e-commerce lead', N'Emergency: Shopify sync stopped', N'The API token had expired on Saturday. Orders from two days were missing.', N'Renew the token; import the missing orders; add an alert if the sync stops.', N'seed'),
(24, 21, '2026-10-20T10:00:00', 90, N'Online', N'Mia Thompson, area managers', N'Dashboard requirements', N'Managers want margin by department and a list of stock-outs every morning.', N'Build mobile dashboards; pilot with two stores.', N'seed'),
(25, 22, '2026-04-20T10:00:00', 120, N'On-site', N'Kasun Wickramasinghe, sales team', N'CRM pipeline workshop', N'Stages agreed: enquiry, site survey, quotation, installation.', N'Configure the pipeline; connect WhatsApp enquiries.', N'seed'),
(26, 23, '2026-06-17T14:00:00', 120, N'On-site', N'Project managers', N'Project costing training', N'Project managers found timesheets easy but materials confusing.', N'Send a short video on recording materials.', N'seed'),
(27, 25, '2026-05-14T10:00:00', 60, N'Online', N'Wei Ling Tan, lab managers', N'Reporting requirements', N'Turnaround time is measured from sample received to result approved.', N'Build saved searches; schedule the Monday email.', N'seed'),
(28, 27, '2026-09-15T09:00:00', 180, N'On-site', N'Chamara Bandara, workshop leads', N'Made-to-order process', N'Sizes and timber type on the sales order must drive the bill of materials.', N'Build a configurator template; test with 5 real orders.', N'seed'),
(29, 27, '2026-10-27T10:00:00', 60, N'Online', N'Chamara Bandara', N'Configurator test', N'Three of five test orders produced the correct cutting list.', N'Fix the drawer and shelf rules.', N'seed'),
(30, 28, '2026-08-06T10:00:00', 90, N'Online', N'Chamara Bandara, accountant', N'Stock valuation investigation', N'Teak was bought in cubic feet but received in pieces, inflating the value.', N'Correct the unit of measure; revalue timber stock.', N'seed'),
(31, 29, '2026-11-27T09:00:00', 240, N'On-site', N'Anjali Gunawardena, finance team, auditors', N'Year-end preparation', NULL, NULL, N'seed'),
(32, 31, '2026-09-02T09:30:00', 240, N'On-site', N'Five new accounts staff', N'Accounts payable training', N'Covered supplier invoices, payment runs and petty cash.', N'Practical assessment next week.', N'seed'),
(33, 32, '2026-07-06T09:00:00', 120, N'On-site', N'Ravi Fernando, HR', N'Leave and attendance setup', N'Fingerprint machines export a CSV file every night.', N'Schedule the daily attendance import.', N'seed'),
(34, 34, '2026-10-16T10:00:00', 90, N'Online', N'Dilani Perera, export team', N'Export document templates', N'Certificates of origin need the HS code and the country of origin for each product.', N'Add HS codes to items; design the document templates.', N'seed');
SET IDENTITY_INSERT dbo.erp_meetings OFF;
GO

SET IDENTITY_INSERT dbo.erp_orders ON;
INSERT INTO dbo.erp_orders (id, client_id, consultation_id, order_date, service, hours, amount, currency, status, notes, created_by) VALUES
(1, 1, 1, '2026-02-10', N'Support hours - Finance', 4, 102000.00, 'LKR', N'Paid', NULL, N'seed'),
(2, 1, 2, '2026-03-09', N'Implementation package - HR & Payroll', 12, 270000.00, 'LKR', N'Paid', NULL, N'seed'),
(3, 2, 3, '2026-02-24', N'Implementation package - Inventory', 16, 456000.00, 'LKR', N'Paid', NULL, N'seed'),
(4, 2, 4, '2026-10-05', N'Customisation - Integration', 24, 792000.00, 'LKR', N'Confirmed', NULL, N'seed'),
(5, 3, 5, '2026-04-14', N'Implementation package - Manufacturing', 20, 540000.00, 'LKR', N'Paid', NULL, N'seed'),
(6, 3, 6, '2026-05-12', N'Training session - Manufacturing', 6, 126000.00, 'LKR', N'Paid', NULL, N'seed'),
(7, 3, 7, '2026-11-03', N'Support hours - Manufacturing', 8, 216000.00, 'LKR', N'Invoiced', NULL, N'seed'),
(8, 4, 8, '2026-03-20', N'Customisation - Finance', 10, 850.00, 'USD', N'Paid', N'Paid late after two reminders.', N'seed'),
(9, 4, 9, '2026-01-26', N'Data migration - Purchasing', 14, 1120.00, 'USD', N'Paid', NULL, N'seed'),
(10, 4, 10, '2026-11-18', N'System health check - Inventory', 6, 570.00, 'USD', N'Confirmed', NULL, N'seed'),
(11, 5, 11, '2026-02-16', N'Data migration - Projects', 30, 720000.00, 'LKR', N'Paid', NULL, N'seed'),
(12, 5, 12, '2026-10-01', N'Customisation - HR & Payroll', 18, 405000.00, 'LKR', N'Invoiced', NULL, N'seed'),
(13, 6, 13, '2026-04-22', N'Support hours - Finance', 5, 425.00, 'USD', N'Invoiced', N'Client disputes the extra hours. Payment on hold.', N'seed'),
(14, 6, 14, '2026-06-01', N'Implementation package - Inventory', 22, 2090.00, 'USD', N'Paid', NULL, N'seed'),
(15, 6, 15, '2026-11-20', N'Training session - Inventory', 4, 280.00, 'USD', N'Confirmed', NULL, N'seed'),
(16, 7, 16, '2026-06-15', N'System health check - Manufacturing', 8, 720.00, 'USD', N'Paid', NULL, N'seed'),
(17, 7, 18, '2026-11-24', N'Version upgrade - Finance', 16, 1360.00, 'USD', N'Confirmed', NULL, N'seed'),
(18, 8, 19, '2026-03-18', N'Implementation package - CRM', 14, 1008.00, 'USD', N'Paid', N'10% discount agreed for the training bundle.', N'seed'),
(19, 8, 20, '2026-07-08', N'Support hours - Integration', 3, 330.00, 'USD', N'Paid', NULL, N'seed'),
(20, 8, 21, '2026-10-20', N'Customisation - Reporting', 12, 960.00, 'USD', N'Confirmed', NULL, N'seed'),
(21, 9, 22, '2026-04-20', N'Implementation package - CRM', 10, 240000.00, 'LKR', N'Paid', NULL, N'seed'),
(22, 9, 23, '2026-06-17', N'Training session - Projects', 4, 84000.00, 'LKR', N'Paid', NULL, N'seed'),
(23, 9, 24, '2026-11-06', N'Support hours - Finance', 3, 76500.00, 'LKR', N'Quoted', N'Urgent support at the standard hourly rate.', N'seed'),
(24, 10, 25, '2026-05-14', N'Customisation - Reporting', 9, 720.00, 'USD', N'Paid', NULL, N'seed'),
(25, 10, 26, '2026-07-01', N'Customisation - HR & Payroll', 3, 225.00, 'USD', N'Cancelled', NULL, N'seed'),
(26, 11, 27, '2026-09-15', N'Implementation package - Manufacturing', 28, 756000.00, 'LKR', N'Invoiced', NULL, N'seed'),
(27, 11, 28, '2026-08-06', N'Support hours - Inventory', 5, 142500.00, 'LKR', N'Paid', NULL, N'seed'),
(28, 12, 29, '2026-11-27', N'Support hours - Finance', 12, 306000.00, 'LKR', N'Confirmed', NULL, N'seed'),
(29, 12, 31, '2026-09-02', N'Training session - Finance', 6, 126000.00, 'LKR', N'Paid', NULL, N'seed'),
(30, 2, 32, '2026-07-06', N'Implementation package - HR & Payroll', 10, 225000.00, 'LKR', N'Paid', NULL, N'seed'),
(31, 1, 34, '2026-10-16', N'Customisation - Sales', 15, 495000.00, 'LKR', N'Invoiced', NULL, N'seed'),
(32, 2, NULL, '2026-01-05', N'Annual support contract', 40, 1020000.00, 'LKR', N'Paid', N'Annual support contract, 40 hours, valid until December 2026.', N'seed'),
(33, 7, NULL, '2026-02-01', N'Annual support contract', 60, 5100.00, 'USD', N'Paid', N'Annual support contract, 60 hours.', N'seed'),
(34, 12, NULL, '2026-11-03', N'Annual support contract', 30, 765000.00, 'LKR', N'Quoted', N'Annual support contract proposal for 2027.', N'seed');
SET IDENTITY_INSERT dbo.erp_orders OFF;
GO

SET IDENTITY_INSERT dbo.erp_feedback ON;
INSERT INTO dbo.erp_feedback (id, consultation_id, submitted_at, rating, comments, would_recommend, follow_up_required, created_by) VALUES
(1, 1, '2026-02-20T10:00:00', 5, N'Tharindu explained everything clearly and our month-end close is now much faster. Very helpful.', 1, 0, N'seed'),
(2, 2, '2026-04-15T09:00:00', 4, N'Payroll works well. The parallel run was stressful but worth it. The payslip layout still needs a small change.', 1, 1, N'seed'),
(3, 3, '2026-03-10T16:00:00', 5, N'Excellent work in the warehouses. Our stock counts finally match the system.', 1, 0, N'seed'),
(4, 5, '2026-05-02T11:00:00', 4, N'Good knowledge of manufacturing. Some bill of materials changes took longer than planned.', 1, 0, N'seed'),
(5, 6, '2026-05-15T09:30:00', 5, N'Supervisors loved the training and the guide in Sinhala was a great idea.', 1, 0, N'seed'),
(6, 8, '2026-04-10T14:00:00', 3, N'The dashboard is useful, but exchange rates were wrong for the first two weeks and we had to chase several times.', 1, 1, N'seed'),
(7, 9, '2026-02-05T10:00:00', 4, N'Smooth migration. The duplicate suppliers were cleaned up properly.', 1, 0, N'seed'),
(8, 11, '2026-03-01T08:00:00', 5, N'Cut-over happened on time even with very messy spreadsheets. Fantastic team.', 1, 0, N'seed'),
(9, 13, '2026-04-30T17:00:00', 2, N'We got the GST return filed, but only after a stressful week. Response was slow at first and we were not happy about the extra cost.', 0, 1, N'seed'),
(10, 14, '2026-06-25T10:00:00', 5, N'Batch tracking is exactly what our auditors wanted.', 1, 0, N'seed'),
(11, 16, '2026-06-30T12:00:00', 4, N'MRP suggestions are realistic now. We would like a follow-up review in six months.', 1, 1, N'seed'),
(12, 19, '2026-04-05T09:00:00', 5, N'The loyalty programme launch went very smoothly in all stores.', 1, 0, N'seed'),
(13, 20, '2026-07-10T10:00:00', 2, N'The fix was quick, but we lost two days of online orders before anyone noticed. Disappointed there was no monitoring in place.', 0, 1, N'seed'),
(14, 22, '2026-05-05T15:00:00', 4, N'No more lost enquiries. The WhatsApp integration is great.', 1, 0, N'seed'),
(15, 23, '2026-06-20T09:00:00', 3, N'Training was fine but too short for our project managers.', 1, 0, N'seed'),
(16, 25, '2026-05-30T10:00:00', 5, N'Reports arrive every Monday without anyone touching them. Very impressed.', 1, 0, N'seed'),
(17, 28, '2026-08-12T11:00:00', 4, N'Found the unit of measure problem quickly. Professional and patient.', 1, 0, N'seed'),
(18, 31, '2026-09-10T16:00:00', 5, N'A very patient trainer. The new staff are confident now.', 1, 0, N'seed'),
(19, 32, '2026-07-20T10:00:00', 3, N'Leave rules work, but the fingerprint import failed twice in the first week.', 1, 1, N'seed'),
(20, 4, '2026-10-30T09:00:00', 2, N'The integration is three weeks late and updates are rare. We are frustrated.', 0, 1, N'seed'),
(21, 12, '2026-10-30T18:00:00', 4, N'Good progress. Wage calculation tests look correct so far.', 1, 0, N'seed');
SET IDENTITY_INSERT dbo.erp_feedback OFF;
GO
