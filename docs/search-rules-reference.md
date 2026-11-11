# Hand-written search rules (readable copy)

Generated from `server-dotnet/SemanticSearch.McpServer/Rules/search-rules.json` — edit that file, not this one.
Patterns run on lower-cased text. `\b` = word edge, `\w*` = any word ending, `(a|b)` = a or b, `?` = optional, `[^.]{0,30}` = up to 30 characters within the same sentence.

## Meaning rules (65 concepts)

A row or question is tagged with a concept when **any one** of its patterns matches.

### 1. Delivery problem  (`delivery_problem`)
- `\b(parcel|package|order|delivery|shipment|item|stuff|it)s?\b[^.]{0,20}\b(never|not|hasn'?t|haven'?t|didn'?t|still not|yet to)\b[^.]{0,15}\b(arriv\w*|come|came|deliver\w*|show(ed|n)? up|turn(ed)? up|here)\b`
- `\b(never|not|didn'?t|haven'?t)\b[^.]{0,15}\b(receiv\w*|got|get)\b[^.]{0,15}\b(parcel|package|order|delivery|item)s?\b`
- `\b(late|delay\w*|missing|lost|stolen|overdue)\b[^.]{0,25}\b(parcel|package|order|delivery|shipment|item)s?\b`
- `\b(parcel|package|order|delivery|shipment)s?\b[^.]{0,25}\b(is |was |are )?(late|delay\w*|missing|lost|overdue)\b`
- `\bwhere('?s| is) my (parcel|package|order|delivery|stuff|item)`
- `\b(tracking (link|number|page|info\w*|has)|track my (parcel|package|order))\b`
- `\bcourier\b`

### 2. Delivery options and cost  (`delivery_options`)
- `\b(shipping|delivery|postage) (cost|fee|charge|price|option|time)s?\b`
- `\b(free|express|next[- ]day|same[- ]day|standard) (shipping|delivery)\b`
- `\bhow (much|long)\b[^.?]{0,25}\b(ship\w*|deliver\w*|postage|arrive)`
- `\bship(s|ping)? free\b`

### 3. Returns and refunds  (`returns_refunds`)
- `\brefund\w*`
- `\bmoney back\b`
- `(?<!(tax|gst|vat) )\b(return|send back|send it back|take back|exchange)\w*\b`

### 4. Changing or cancelling an order  (`change_cancel_order`)
- `(?<!noise[- ])\bcancel\w*`
- `\b(change|modify|edit|amend)\b[^.]{0,15}\border`
- `\bchanged my mind\b`

### 5. Warranty and cover  (`warranty`)
- `\bwarrant(y|ies)\b`
- `\bguarantee\w*`
- `\bprotection plans?\b`
- `\b(extended|insurance|accidental damage)\b`

### 6. Broken or faulty device  (`fault_repair`)
- `\b(broken|broke|faulty|defect\w*|malfunction\w*|cracked|repair\w*)\b`
- `\b(stopped working|not working|doesn'?t work|does not work|won'?t (turn on|work|start|switch on)|dead)\b`
- `\bstops working\b`

### 7. Screen or picture problem  (`screen_problem`)
- `\b(no|black|blank|dark) (picture|screen|display|image)\b`
- `\b(screen|display|picture)\b[^.]{0,20}\b(black|blank|dark|flicker\w*|stays off)\b`
- `\bbacklight\b`
- `\bsound but no picture\b`

### 8. Bluetooth and pairing  (`bluetooth`)
- `\bbluetooth\b`
- `\bpair(ing|ed|s)?\b`
- `\bwon'?t connect\b`
- `\b(earbuds?|headphones?|speakers?)\b[^.]{0,25}\bconnect\w*`

### 9. Wi-Fi and internet  (`internet`)
- `\bwi-?fi\b`
- `\binternet\b`
- `\brouters?\b`
- `\bmesh\b`
- `\bextender\b`
- `\bdead zones?\b`
- `\b(signal|connection)\b[^.]{0,15}\b(drop\w*|weak|slow|bad)\b`
- `\bweak signal\b`

### 10. Battery and charging  (`battery`)
- `\bbatter(y|ies)\b`
- `\b(re)?charg(e|er|ers|es|ing)\b`
- `\bpower bank\b`
- `\b(dies|died|drains?|drained|draining|runs? out)\b[^.]{0,15}\b(fast|quick\w*|soon)\b`

### 11. Overheating  (`overheating`)
- `\b(overheat\w*|too hot|very hot|gets hot|getting hot|warm|burning up)\b`
- `\btemperature warning\b`

### 12. Password and sign-in  (`sign_in`)
- `\bpasswords?\b`
- `\b(log ?in|login|sign ?in|signing in|locked out|can'?t (get in|access))\b`

### 13. Account security  (`account_security`)
- `\b(two[- ]factor|2fa|mfa|authenticat\w*|hack\w*|security code|backup codes?)\b`

### 14. Delivery address  (`address`)
- `\baddress(es)?\b`
- `\bmoved house\b`
- `\bwrong (house|street)\b`

### 15. Ways to pay  (`payment`)
- `\b(payment methods?|credit card|debit card|visa|mastercard|american express|paypal|apple pay|google pay|checkout)\b`
- `\b(how|can i|ways? to) pay\b`

### 16. Paying monthly  (`instalments`)
- `\binstal?ments?\b`
- `\bmonthly (payments?|plan|parts)\b`
- `\bpay (monthly|in parts|over time|later)\b`
- `\bin (monthly )?parts\b`
- `\bsplit\b[^.]{0,15}\b(payment|cost|price)\b`
- `\binterest[- ]free\b`
- `\b(finance|credit check)\b`

### 17. Billing problem  (`billing_error`)
- `\b(charged|billed|paid)\b[^.]{0,10}\b(twice|two times|double|again|too much)\b`
- `\b(double|duplicate) (charge|payment)s?\b`
- `\bauthori[sz]ation hold\b`
- `\bwrong (amount|charge)\b`

### 18. Discounts and deals  (`discounts`)
- `\b(discounts?|cheaper|price match\w*|deals?|sale|promo\w*|coupons?|offers?|percent off|save money)\b`

### 19. Students  (`students`)
- `\b(students?|universit(y|ies)|college|school|education|campus|dorm\w*)\b`

### 20. Children  (`children`)
- `\b(kids?|child|children|teen\w*|parental|my son|my daughter)\b`

### 21. Travel and on the go  (`travel`)
- `\b(travel\w*|trips?|flights?|planes?|airline|aeroplane|airplane|holiday|vacation|commut\w*|on the go|portable|carry-on|camping|campsite|beach)\b`

### 22. Blocking out noise  (`noise_blocking`)
- `\bnoise[- ]cancel\w*`
- `\bblock\w*\b[^.]{0,10}\b(noise|sound|engine)\b`
- `\b(noisy|engine hum|chatter|too loud)\b`

### 23. Light and compact  (`lightweight`)
- `\b(light ?weight|light enough|lighter|thin|compact|slim|palm-sized|small)\b`
- `\bweigh(s|ing)?\b`

### 24. Gaming  (`gaming`)
- `\b(gam(e|es|ing|er|ers)|consoles?|playstation|xbox|rtx)\b`
- `\b1(20|44|65)hz\b`

### 25. Work and study  (`work_study`)
- `\b(home office|office|spreadsheets?|typing|professionals?|productivity|working days?|studying|homework|note taking|essays?)\b`

### 26. Fitness and health  (`fitness`)
- `\b(fitness|workouts?|gym|jogging|runners?|exercis\w*|heart rate|sleep tracking|sleep analysis|swim\w*|blood oxygen|steps)\b`

### 27. Home security cameras  (`home_security`)
- `\b(security cameras?|cameras?|doorbells?|cctv|intruders?|burglar\w*|person detection|night vision|driveway)\b`

### 28. Privacy  (`privacy`)
- `\b(privacy|private|encrypt\w*|my data|footage|recordings?|spying)\b`

### 29. Cleaning  (`cleaning`)
- `\b(vacuum\w*|hoover\w*|cleaning|pet hair|mop\w*|suction)\b`

### 30. Air quality and allergies  (`air_quality`)
- `\b(allerg\w*|pollen|air quality|air purif\w*|smoke|hepa|asthma|dust)\b`

### 31. Heating and energy bills  (`heating_energy`)
- `\b(heating|thermostat|boiler|heat pump|radiators?)\b`
- `\benergy (bills?|saving|costs?)\b`
- `\b(cold|freezing) (house|home|room)\b`

### 32. Recycling and trade-in  (`recycling_tradein`)
- `\b(recycl\w*|e-waste|dispose|get rid of|trade[- ]in|trading in)\b`
- `\bold (phone|laptop|tv|device|electronics|tablet)s?\b`

### 33. Gifts  (`gifts`)
- `\b(gift\w*|presents?|birthday|vouchers?)\b`

### 34. Contacting support  (`contact_support`)
- `\b(contact (you|us|support)|customer (service|support)|live chat|phone number|(speak|talk) to (someone|a person|support|an agent|a human|customer service)|opening hours|support team|real person)\b`
- `\bcontacting customer support\b`

### 35. Error code  (`error_code`)
- `\berror\b`
- `\b[a-z]-?\d{3}\b`

### 36. Installation and setup  (`installation`)
- `\b(install\w*|set ?up|setting up|mount\w*|brackets?|vesa|on the wall|hang (it|the tv))\b`

### 37. TV and films  (`tv_film`)
- `\b(tvs?|televisions?|films?|movies?|cinema|netflix|streaming|dialogue|hdr|4k|oled|qled)\b`

### 38. Music and sound  (`music_audio`)
- `\b(music|songs?|audio|speakers?|bass|surround|dolby atmos|soundbar|headphones?|earbuds?|listen\w*)\b`

### 39. Phones and tablets  (`phones_tablets`)
- `\b(phones?|smartphones?|mobile|tablets?|ipad|android)\b`

### 40. Computers  (`computers`)
- `\b(laptops?|computers?|pc|desktop|notebook|ultrabook|monitors?|keyboards?|mouse)\b`

### 41. Affordable  (`budget`)
- `\b(cheap\w*|budget|affordable|inexpensive|low[- ]cost|not too expensive)\b`
- `\bunder \$?\d+\b`

### 42. Comfort and ergonomics  (`comfort`)
- `\b(ergonomic\w*|wrist|strain|pain|comfort\w*|posture)\b`

### 43. Finance and accounting  (`erp_finance`)
- `\b(general ledger|ledgers?|journals?|journal entr\w*|accounts? (payable|receivable)|chart of accounts|accruals?|depreciation|fixed assets?|balance sheet|fiscal)\b`
- `\b(month|year)[- ]end\b`
- `\bclos(e|ing) the (month|year|books)\b`
- `\b(bank )?reconcil\w*`
- `\baudit\w*`

### 44. Tax and VAT  (`erp_tax`)
- `\b(tax\w*|vat|gst|withholding)\b`

### 45. Invoices and billing  (`erp_invoicing`)
- `\binvoic\w*`
- `\bcredit notes?\b`
- `\b(customer|client)s? (are )?waiting for (their|the) bills?\b`

### 46. Inventory and stock  (`erp_inventory`)
- `\b(inventory|stock\w*|warehouses?|bin locations?|cycle count\w*|stock ?takes?|stock count\w*|reorder\w*|batch\w*|expiry|serial numbers?|goods receipt|picking|stock transfers?)\b`

### 47. Purchasing and suppliers  (`erp_purchasing`)
- `\b(purchas\w*|procurement|suppliers?|vendors?|purchase orders?)\b`

### 48. Sales and CRM  (`erp_sales`)
- `\b(sales orders?|quotations?|crm|leads?|pipeline|opportunit\w*|loyalty|customers? portal|enquir\w*)\b`

### 49. HR and payroll  (`erp_payroll`)
- `\b(payroll|payslips?|salar\w*|wages?|overtime|allowances?|epf|etf|leave (rules|policies|balance)|annual leave|attendance|employees?|hr)\b`

### 50. Manufacturing and production  (`erp_manufacturing`)
- `\b(manufactur\w*|production|bills? of materials?|boms?|work orders?|routings?|mrp|material requirements|shop floor|made[- ]to[- ]order|cutting list)\b`

### 51. Reports and dashboards  (`erp_reporting`)
- `\b(reports?|reporting|dashboards?|kpis?|analytics|saved searches|power bi)\b`

### 52. Integration and syncing  (`erp_integration`)
- `\b(integrat\w*|api|apis|sync\w*|webhooks?|shopify|woocommerce|e-?commerce|edi|cad system|tracking api|connect(ed|ing)? (to|with) )\b`
- `\b(orders?|data) (stopped|not) (appearing|coming|syncing)\b`

### 53. Data migration  (`erp_migration`)
- `\b(migrat\w*|data import|import(ed|ing)? (the )?data|legacy (system|data|package)|opening balances|cleans\w*|cut-?over|spreadsheets?|old (accounting|system|package))\b`

### 54. Training  (`erp_training`)
- `\b(train\w*|workshops?|onboard\w*|user guides?|reference guides?|how to use|knowledge transfer|teach\w*|learn\w*)\b`

### 55. Slow or freezing system  (`erp_performance`)
- `\b(slow\w*|performance|freez\w*|lag\w*|time ?outs?|hangs?|hanging|takes? (ages|forever|minutes|almost a minute)|too long)\b`

### 56. Upgrades  (`erp_upgrade`)
- `\b(upgrad\w*|new version|latest version|end of support|patch\w*)\b`

### 57. Go-live  (`erp_go_live`)
- `\b(go-?live|going live|launch\w*|parallel run|hypercare)\b`

### 58. Errors and failures  (`erp_errors`)
- `\b(errors?|bugs?|crash\w*|fail(s|ed|ing|ure)?|stuck|cannot be (validated|posted)|won'?t post|not posting|wrong|incorrect|do(es)? not match|mismatch\w*)\b`

### 59. Customisation  (`erp_customisation`)
- `\b(customi[sz]\w*|custom (fields?|reports?|forms?|workflows?|templates?)|templates?|configurator|workflows?|approvals?)\b`

### 60. Delays and missed deadlines  (`erp_delay`)
- `\b(late|delay\w*|behind schedule|overdue|missed (the )?deadline|weeks? late|running late|took longer than planned|took longer)\b`

### 61. Urgent  (`erp_urgent`)
- `\b(urgent\w*|asap|as soon as possible|critical|emergency|immediately|blocking|due next week|customers are waiting)\b`

### 62. Unhappy client  (`client_unhappy`)
- `\b(unhappy|disappoint\w*|frustrat\w*|dissatisf\w*|not (happy|satisfied|impressed|pleased)|poor|terrible|complain\w*|angry|let down|chase\w*|slow to respond|response was slow|updates are rare)\b`
- `\b(would not|wouldn'?t|won'?t|not) recommend\b`
- `\bbad (service|experience)\b`
- `\bextra cost\b`

### 63. Happy client  (`client_happy`)
- `\b(excellent|fantastic|great|loved|delighted|impress\w*|very helpful|professional|patient|knowledgeable|smooth\w*|exactly what|confident now|thank\w*)\b`
- `(?<!not |n't )\brecommend\w*\b`
- `\b(happy|satisfied) (client|customer)s?\b`

### 64. Follow-up needed  (`follow_up`)
- `\bfollow[- ]?ups?\b`
- `\b(next steps?|action items?|still needs?|needs a small change|review (in|again))\b`

### 65. Payments and money owed  (`money_owed`)
- `\b(unpaid|outstanding|overdue payments?|paid late|reminders?|disput\w*|payment on hold|not (yet )?paid|owe\w*)\b`

## Synonyms (27 groups)

Each word on the right is compared as the word on the left.

| Counts as | Words |
|---|---|
| **parcel** | parcel, package, shipment, delivery |
| **tv** | tv, television, telly |
| **laptop** | laptop, notebook, ultrabook |
| **phone** | phone, smartphone, mobile, cellphone |
| **headphone** | headphone, headset, earphone |
| **earbud** | earbud, earpiece |
| **broken** | broken, faulty, defective, damaged |
| **repair** | repair, fix, mend |
| **cheap** | cheap, affordable, inexpensive, budget |
| **wifi** | wifi, wi-fi |
| **password** | password, passcode |
| **child** | child, children, kid |
| **price** | price, cost, fee |
| **fast** | fast, quick, rapid |
| **large** | large, big, huge |
| **small** | small, compact, mini, tiny |
| **vacuum** | vacuum, hoover |
| **tablet** | tablet, ipad |
| **email** | email, e-mail |
| **buy** | buy, purchase |
| **arrive** | arrive, come, came |
| **supplier** | supplier, vendor |
| **employee** | employee, staff, worker |
| **client** | client, customer |
| **invoice** | invoice, bill |
| **consultation** | consultation, session, booking |
| **consultant** | consultant, advisor, adviser |

## Stop words (91)

Ignored when comparing words:

a, an, and, are, as, at, be, been, but, by, can, could, do, does, did, for, from, get, got, how, i, i'm, if, in, into, is, it, it's, its, me, my, of, on, or, our, so, that, the, their, them, there, this, to, too, up, us, was, we, what, when, where, which, who, why, will, with, would, you, your, yet, has, have, had, just, any, some, also, than, then, very, all, one, need, want, should, while, really, keeps, there's, about, after, before, out, off, over, under, more, most, only, own, same

## Stemming (applied to every word, in this order)

| Pattern | Replace with | Example |
|---|---|---|
| `^(\w{3,})ies$` | `$1y` | batteries → battery |
| `^(\w{3,}?)(ing|ed|ly)$` | `$1` | charging → charg, delivered → deliver |
| `^(\w{2,}[^s])s$` | `$1` | orders → order |
| `^(\w{3,})e$` | `$1` | charge → charg |
| `^(\w{2,}?)([bdfglmnprt])\2$` | `$1$2` | cancell → cancel |

## Chat rules (`backend/app/chat.py`)

Checked top to bottom; the first match decides the tool. Anything that matches nothing becomes a meaning search.

| # | Message contains | Tool |
|---|---|---|
| 1 | text between slashes, e.g. `/E-\d{3}/` | `semantic_search` (pattern mode) |
| 2 | `consultation|booking|case|job` + a number | `get_consultation` |
| 3 | `workload|how busy|busiest|capacity|utilisation` | `consultant_workload` |
| 4 | `revenue|income|earnings|turnover|billing`, or `orders/sales report/summary/totals` | `revenue_report` |
| 5 | `satisfaction|average rating|ratings report|feedback report` | `feedback_report` |
| 6 | `how many consultations|consultations report/summary/by/per` | `consultation_report` |
| 7 | `meeting(s)` + `upcoming|next|planned|this week…` or a list word | `list_meetings` |
| 8 | `feedback|rating|review|comments` + `bad|poor|good|follow-up` or a list word | `list_feedback` |
| 9 | `order|quote|invoice` + a list word or `unpaid|paid|quoted|invoiced…` | `list_orders` |
| 10 | `consultants` / `clients` + a list word or `who` | `list_consultants` / `list_clients` |
| 11 | `consultation|booking|job` + a list word, a status, `open` or `urgent` | `list_consultations` |
| 12 | anything else | `semantic_search` (meaning) |

List words: `list, show, display, give me, get, find, which, what, all, any, see, view`.

| Words | Become |
|---|---|
| requested, new, pending | status Requested |
| scheduled, booked, planned | status Scheduled |
| in progress, ongoing, active | status In progress |
| completed, finished, done, closed | status Completed |
| cancelled, canceled | status Cancelled |
| open, unfinished, outstanding, current | Requested + Scheduled + In progress |
| urgent, critical, high priority | priority Urgent |
| hr, payroll, salary, wages | module HR & Payroll |
| finance, accounting, accounts, ledger | module Finance |
| inventory, stock, warehouse | module Inventory |
| manufacturing, production | module Manufacturing |
| purchasing, procurement | module Purchasing |
| integration, api | module Integration |
| reporting, dashboard | module Reporting |
| crm | module CRM |
| project(s) | module Projects |
| for/from/of/with + Capitalised Name | client filter |
| by/per + month, client, consultant… | report grouping |
| unpaid, outstanding, owed | orders Confirmed + Invoiced |
| bad, poor, low, negative, unhappy (feedback) | rating ≤ 2 |
| good, great, best, positive, happy (feedback) | rating ≥ 4 |
