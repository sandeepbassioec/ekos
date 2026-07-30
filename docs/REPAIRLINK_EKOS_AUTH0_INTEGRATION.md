# RepairLink + EKOS/VectorVolt: Auth0 Integration Strategy

**Version:** 1.0  
**Date:** 2026-07-12  
**Purpose:** Unified authentication and data strategy for RepairLink integration into EKOS/VectorVolt ecosystem

> **Vision:** One identity system (Auth0) controls access across RepairLink, EKOS, and VectorVolt. Data flows seamlessly between systems while maintaining security and multi-tenancy isolation.

---

## Table of Contents

1. [Executive Summary](#executive-summary)
2. [Strategic Rationale](#strategic-rationale)
3. [Architecture Overview](#architecture-overview)
4. [Auth0 Setup & Configuration](#auth0-setup--configuration)
5. [Multi-Tenancy Model](#multi-tenancy-model)
6. [RepairLink Integration](#repairlink-integration)
7. [EKOS Integration](#ekos-integration)
8. [Data Flow & APIs](#data-flow--apis)
9. [Role-Based Access Control (RBAC)](#role-based-access-control-rbac)
10. [Security Model](#security-model)
11. [Implementation Roadmap](#implementation-roadmap)
12. [Example Workflows](#example-workflows)

---

## Executive Summary

### The Problem

**Current State:**
- RepairLink: Standalone system with its own auth
- EKOS/VectorVolt: Separate system with separate auth
- No unified identity
- Data silos between systems
- Complex multi-system user management

### The Solution

**Auth0 as Central Identity Hub:**

```
┌─────────────────────────────────────────┐
│         Auth0 (Identity Hub)            │
│  - User authentication                  │
│  - Authorization/RBAC                   │
│  - Multi-tenancy                        │
│  - MFA, SSO, social login               │
└─────────────────────────────────────────┘
    ↑              ↑              ↑
    │              │              │
    ▼              ▼              ▼
┌──────────┐  ┌──────────┐  ┌──────────┐
│RepairLink│  │EKOS Core │  │VectorVolt│
│(Nodejs) │  │(React)   │  │(Rust)    │
└──────────┘  └──────────┘  └──────────┘
    │              │              │
    └──────────────┴──────────────┘
          Unified Data Layer
       (VectorVolt + APIs)
```

### Benefits

✅ **Single Sign-On (SSO)** - Log in once, access all systems  
✅ **Unified RBAC** - One place to manage permissions  
✅ **Multi-Tenancy** - Org A's users isolated from Org B's  
✅ **MFA/Security** - Enterprise-grade authentication  
✅ **Audit Trail** - All logins and access logged  
✅ **User Management** - Central user directory  
✅ **Social Login** - Google, Microsoft, etc.  
✅ **API Keys** - For service-to-service auth  

---

## Strategic Rationale

### Why Auth0?

**1. Enterprise Standard**
- Used by 1000+ enterprise companies
- Battle-tested, compliant (SOC2, HIPAA, GDPR)
- Handles millions of logins daily

**2. Multi-Tenancy Built-in**
- Organizations (tenants) are first-class
- Role-based access control (RBAC) per tenant
- Permissions inheritance and delegation

**3. APIs for Everything**
- Management API (control users, roles, permissions)
- Authentication API (login, logout, MFA)
- Authorization API (check permissions)
- No vendor lock-in (standard OAuth2/OIDC)

**4. Ecosystem Integration**
- Works with Node.js (RepairLink)
- Works with React (EKOS UI)
- Works with Rust (VectorVolt via APIs)
- Pre-built SDKs and libraries

**5. Self-Healing Alignment**
- Auth0 can manage permissions dynamically
- As EKOS features deploy, Auth0 permissions update automatically
- No manual permission management

### Why NOT Use Auth0?

❌ Cost (scales with users + features)  
❌ Dependency on external service  
❌ Cannot be modified (black box)  
❌ Latency (network round-trip for auth checks)  

**For EKOS:** Build custom auth in bare-metal OS later if needed

---

## Architecture Overview

### High-Level System Architecture

```
┌─────────────────────────────────────────────────────────┐
│                    Auth0 Cloud Service                  │
│  (Managed, secure, replicated, DDoS protected)          │
│                                                         │
│  ├─ User Database                                      │
│  ├─ Organization/Tenant Management                     │
│  ├─ Role-Based Access Control (RBAC)                   │
│  ├─ API Keys & Service Principals                      │
│  ├─ Audit Logs                                         │
│  └─ Multi-Factor Authentication (MFA)                  │
└─────────────────────────────────────────────────────────┘
    ↓                    ↓                    ↓
    │                    │                    │
    ▼                    ▼                    ▼
┌──────────────┐  ┌──────────────┐  ┌──────────────┐
│ RepairLink   │  │ EKOS UI      │  │ VectorVolt   │
│ (Node.js)    │  │ (React)      │  │ (Rust API)   │
│              │  │              │  │              │
│ ├─ Passport  │  │ ├─ @auth0/   │  │ ├─ jsonwebtoken
│ │  (OAuth2)  │  │ │  react-auth│  │ │  validation  
│ ├─ Login     │  │ │  0-spa     │  │ ├─ Bearer token
│ │  routes    │  │ ├─ Login UI  │  │ │  in header
│ ├─ User info │  │ ├─ User info │  │ ├─ API key
│ │  endpoint  │  │ │  (useAuth0)│  │ │  validation
│ └─ API auth  │  │ └─ Protected │  │ └─ Service
│              │  │    routes    │  │   principal
└──────────────┘  └──────────────┘  └──────────────┘
    ↓                    ↓                    ↓
    └─────────────────────┬────────────────────┘
                          │
                ┌─────────▼─────────┐
                │  Unified API      │
                │  (GraphQL/REST)   │
                │                   │
                │ ├─ RepairLink API │
                │ ├─ EKOS API       │
                │ └─ VectorVolt API │
                │                   │
                │ Auth checks:      │
                │ ├─ JWT validation │
                │ ├─ Org isolation  │
                │ └─ RBAC enforcing │
                └─────────┬─────────┘
                          │
                ┌─────────▼─────────┐
                │   VectorVolt      │
                │  Data Engine      │
                │                   │
                │ ├─ Tenants table  │
                │ ├─ Users table    │
                │ ├─ Roles table    │
                │ ├─ Permissions    │
                │ ├─ RepairLink data│
                │ └─ EKOS data      │
                └───────────────────┘
```

---

## Auth0 Setup & Configuration

### Auth0 Organization Structure

```
Auth0 Tenant
├─ Organization: OEC (Your company)
│  ├─ Members (OEC employees)
│  └─ Roles:
│     ├─ admin (full access)
│     ├─ engineer (EKOS/VectorVolt)
│     └─ support (RepairLink only)
│
├─ Organization: Customer A (Honda dealer)
│  ├─ Members (Honda staff)
│  ├─ Roles:
│  │  ├─ manager
│  │  ├─ technician
│  │  └─ viewer
│  └─ Permissions:
│     ├─ read:repair_orders
│     ├─ write:repair_orders
│     └─ read:inventory
│
└─ Organization: Customer B (Toyota dealer)
   ├─ Members (Toyota staff)
   └─ Roles & Permissions (similar to Customer A)
```

### Auth0 Configuration Steps

**Step 1: Create Auth0 Tenant**

```
Auth0 Dashboard:
  - Create new tenant: vectorvolt
  - Select region: US (or appropriate)
  - Email: admin@vectorvolt.com
```

**Step 2: Configure Applications**

```
For RepairLink (Node.js):
  - Application Type: Regular Web App
  - Name: RepairLink
  - Allowed Callback URLs: https://repairlink.com/callback
  - Allowed Logout URLs: https://repairlink.com/
  - Token Endpoint Auth Method: POST
  
For EKOS UI (React SPA):
  - Application Type: Single Page App
  - Name: EKOS UI
  - Allowed Callback URLs: https://ekos.vectorvolt.io/callback
  - Allowed Logout URLs: https://ekos.vectorvolt.io/
  - Allowed Web Origins: https://ekos.vectorvolt.io
  
For VectorVolt API (Machine-to-Machine):
  - Application Type: Machine to Machine
  - Name: VectorVolt API
  - Grant Type: Client Credentials
  - Audience: https://api.vectorvolt.io
```

**Step 3: Configure APIs**

```
API 1: RepairLink API
  - Name: RepairLink API
  - Identifier: https://api.repairlink.com
  - Signing Algorithm: RS256
  
  Scopes:
  - read:repair_orders
  - write:repair_orders
  - read:inventory
  - write:inventory
  - read:customers
  
API 2: EKOS API
  - Name: EKOS API
  - Identifier: https://api.ekos.vectorvolt.io
  - Signing Algorithm: RS256
  
  Scopes:
  - read:features
  - write:features
  - read:vectorvolt
  - write:vectorvolt
  
API 3: VectorVolt API
  - Name: VectorVolt API
  - Identifier: https://api.vectorvolt.io
  - Signing Algorithm: RS256
  
  Scopes:
  - read:data
  - write:data
  - admin:database
```

**Step 4: Configure Rules (Custom Logic)**

```javascript
// Rule: Add organization context to token
function addOrgContext(user, context, callback) {
  const org = context.org;
  
  // Get organization info from Auth0
  auth0.users.get({id: user.user_id}, function(err, user_info) {
    // Add org to token
    context.idToken['org_id'] = org.id;
    context.idToken['org_name'] = org.display_name;
    
    // Add user roles in org
    const roles = (user_info.app_metadata || {}).roles || [];
    context.idToken['roles'] = roles;
    
    callback(null, user, context);
  });
}
```

---

## Multi-Tenancy Model

### Tenant Isolation Strategy

**Auth0 Organizations (Recommended)**

```
Each customer = separate organization in Auth0

Benefits:
  ✓ Built-in tenant isolation
  ✓ Separate role management per org
  ✓ Permissions scoped to org
  ✓ User directory per org
  ✓ Audit logs per org
  
Structure:

Auth0 Organization "customer_a"
├─ Members:
│  ├─ john@customera.com (role: manager)
│  ├─ jane@customera.com (role: technician)
│  └─ bob@customera.com (role: viewer)
├─ Roles:
│  ├─ manager (can approve orders, manage users)
│  ├─ technician (can create/update orders)
│  └─ viewer (read-only)
└─ Connections:
   ├─ Username-Password (local accounts)
   └─ Google (federated identity)
```

**VectorVolt Data Model**

```
Table: tenants
  id: UUID
  name: String
  auth0_org_id: String (reference to Auth0 org)
  created_at: Timestamp
  
Table: users
  id: UUID
  tenant_id: UUID (foreign key to tenants)
  auth0_user_id: String
  email: String
  name: String
  
Table: roles
  id: UUID
  tenant_id: UUID
  name: String ("admin", "manager", "technician")
  permissions: JSON (list of permission strings)
  
Table: user_roles
  user_id: UUID
  role_id: UUID
  
Table: repair_links
  id: UUID
  tenant_id: UUID
  ... (RepairLink-specific columns)
  
Table: ekos_features
  id: UUID
  tenant_id: UUID
  ... (EKOS-specific columns)
```

### Isolation Enforcement

**In API Layer:**

```rust
// VectorVolt API (pseudocode)

pub async fn get_repair_order(
    user: AuthUser,  // From JWT token
    order_id: UUID,
) -> Result<RepairOrder> {
    // 1. Verify user's org matches order's tenant
    let order = db.repair_orders.get(order_id)?;
    
    if order.tenant_id != user.tenant_id {
        return Err(Error::Forbidden);  // Cross-org access denied
    }
    
    // 2. Check user's permissions for this action
    let user_roles = db.user_roles.get_for_user(user.id)?;
    let perms = get_permissions_from_roles(&user_roles);
    
    if !perms.contains("read:repair_orders") {
        return Err(Error::Unauthorized);  // User can't read orders
    }
    
    // 3. Return order (now safe)
    Ok(order)
}
```

---

## RepairLink Integration

### Current State

**RepairLink Architecture:**
- Node.js backend
- React frontend
- MySQL database
- Custom authentication (likely username/password)
- Direct database access

### Integration Path

**Phase 1: Add Auth0 Authentication (Week 1-2)**

```
Goal: Use Auth0 for login, keep everything else

Steps:
1. Install @auth0/express-openid-connect in Node.js
2. Configure passport with Auth0
3. Redirect login → Auth0 login page
4. Get JWT token from Auth0
5. Store user info in local session
6. Keep existing database and API structure
```

**Phase 2: Implement RBAC (Week 3-4)**

```
Goal: Use Auth0 roles/permissions instead of local auth

Steps:
1. Create Auth0 roles (manager, technician, viewer)
2. Assign roles to users in Auth0
3. Extract roles from JWT token
4. Replace local permission checks with Auth0 roles
5. Sync user list from Auth0 → RepairLink DB
```

**Phase 3: Enable Multi-Tenancy (Week 5-6)**

```
Goal: Each customer's data isolated

Steps:
1. Create Auth0 organization per customer
2. Add org_id to JWT token (from Auth0 organization)
3. Filter all queries by org_id
4. Prevent cross-org access
5. Implement tenant-specific reports/dashboards
```

### RepairLink Code Example

**Before (Auth0 Integration):**

```javascript
// routes/auth.js
app.post('/login', async (req, res) => {
  const {email, password} = req.body;
  
  // Check against local DB
  const user = await User.findOne({email});
  if (!user || !user.password_match(password)) {
    return res.status(401).json({error: 'Invalid credentials'});
  }
  
  // Create session
  req.session.user = user;
  res.redirect('/dashboard');
});
```

**After (Auth0 Integration):**

```javascript
// routes/auth.js
const { auth } = require('express-openid-connect');

// Auth0 middleware
app.use(auth({
  secret: process.env.AUTH0_SECRET,
  baseURL: process.env.BASE_URL,
  clientID: process.env.AUTH0_CLIENT_ID,
  clientSecret: process.env.AUTH0_CLIENT_SECRET,
  issuerBaseURL: process.env.AUTH0_ISSUER_URL,
  authorizationParams: {
    organization: req.query.org,  // Optional org context
  },
}));

// Login - redirects to Auth0
app.get('/login', (req, res) => {
  res.openidConnect.login({
    returnTo: '/dashboard',
  });
});

// Callback - Auth0 redirects here after login
app.get('/callback', (req, res) => {
  // Auth0 middleware automatically handles this
  // Sets req.oidc.user with user info
  res.redirect('/dashboard');
});

// Logout
app.get('/logout', (req, res) => {
  res.openidConnect.logout({
    returnTo: process.env.AUTH0_LOGOUT_URL,
  });
});

// Protected route with RBAC check
app.post('/repair-orders', isAuthenticated, requireRole('technician'), async (req, res) => {
  const {order_data} = req.body;
  const user = req.oidc.user;
  
  // User is authenticated and has 'technician' role
  const order = await RepairOrder.create({
    ...order_data,
    tenant_id: user['org_id'],  // From Auth0 organization
    created_by: user.sub,
  });
  
  res.json(order);
});

// Middleware: Check if user has role
function requireRole(role) {
  return (req, res, next) => {
    const userRoles = req.oidc.user['roles'] || [];
    
    if (!userRoles.includes(role)) {
      return res.status(403).json({error: 'Insufficient permissions'});
    }
    
    next();
  };
}

// Middleware: Check authentication
function isAuthenticated(req, res, next) {
  if (!req.oidc.isAuthenticated()) {
    return res.redirect('/login');
  }
  next();
}
```

---

## EKOS Integration

### Current State

**EKOS Architecture:**
- React frontend (SPA)
- Node.js API backend (generated features)
- Custom authentication (probably token-based)
- Pod Manager (feature execution)

### Integration Path

**Phase 1: Add Auth0 Login to EKOS UI (Week 1-2)**

```
Goal: EKOS users login via Auth0 instead of local auth

Steps:
1. Install @auth0/auth0-react in React
2. Wrap app with Auth0Provider
3. Add login button → redirects to Auth0
4. Get JWT token after login
5. Store token in localStorage
6. Use token for API calls
```

**Phase 2: Secure EKOS API (Week 3-4)**

```
Goal: EKOS API validates JWT tokens from Auth0

Steps:
1. Install jsonwebtoken in Node.js backend
2. Add middleware to validate JWT token
3. Extract user/org from token
4. Check user's permissions (from Auth0 roles)
5. Return 401 if no valid token
6. Return 403 if user lacks permissions
```

**Phase 3: Dynamic Feature Permissions (Week 5-6)**

```
Goal: EKOS features respect user's role/permissions

Steps:
1. Create feature-level permissions in Auth0
   - Example: "generate:filters" role
2. Before generating feature, check if user has permission
3. Before deploying feature, check org quota/limits
4. Track which user generated which feature (audit)
5. Show only permitted features in EKOS UI
```

### EKOS Code Example

**React Frontend:**

```jsx
// App.jsx
import { Auth0Provider, useAuth0 } from "@auth0/auth0-react";

function App() {
  return (
    <Auth0Provider
      domain={process.env.REACT_APP_AUTH0_DOMAIN}
      clientId={process.env.REACT_APP_AUTH0_CLIENT_ID}
      redirectUri={window.location.origin}
      audience={process.env.REACT_APP_AUTH0_AUDIENCE}
      scope="openid profile email"
    >
      <Dashboard />
    </Auth0Provider>
  );
}

function Dashboard() {
  const { user, isAuthenticated, loginWithRedirect, logout, getAccessTokenSilently } = useAuth0();
  const [features, setFeatures] = useState([]);
  
  if (!isAuthenticated) {
    return (
      <div>
        <button onClick={() => loginWithRedirect()}>
          Login with Auth0
        </button>
      </div>
    );
  }
  
  useEffect(() => {
    // Fetch features with JWT token
    const fetchFeatures = async () => {
      const token = await getAccessTokenSilently();
      
      const response = await fetch('/api/features', {
        headers: {
          Authorization: `Bearer ${token}`,
        },
      });
      
      const data = await response.json();
      setFeatures(data);
    };
    
    fetchFeatures();
  }, [getAccessTokenSilently]);
  
  return (
    <div>
      <h1>Welcome, {user.name}</h1>
      <p>Organization: {user['org_name']}</p>
      
      <FeatureGenerator />
      
      <FeatureList features={features} />
      
      <button onClick={() => logout({returnTo: window.location.origin})}>
        Logout
      </button>
    </div>
  );
}

function FeatureGenerator() {
  const { user, getAccessTokenSilently } = useAuth0();
  const [prompt, setPrompt] = useState('');
  
  const handleGenerate = async () => {
    const token = await getAccessTokenSilently();
    
    const response = await fetch('/api/features/generate', {
      method: 'POST',
      headers: {
        'Content-Type': 'application/json',
        Authorization: `Bearer ${token}`,
      },
      body: JSON.stringify({
        intent: prompt,
        tier: 1,
      }),
    });
    
    if (response.status === 403) {
      alert('You do not have permission to generate features');
      return;
    }
    
    const feature = await response.json();
    alert(`Feature "${feature.name}" generated and deployed!`);
  };
  
  return (
    <div>
      <textarea
        value={prompt}
        onChange={(e) => setPrompt(e.target.value)}
        placeholder="Describe the feature you want..."
      />
      <button onClick={handleGenerate}>Generate Feature</button>
    </div>
  );
}
```

**Node.js Backend:**

```javascript
// middleware/auth.js
const jwt = require('jsonwebtoken');
const jwksClient = require('jwks-rsa');

// Cache JWKS
const jwksCache = {};

const getSigningKey = async (kid) => {
  const client = jwksClient({
    jwksUri: `${process.env.AUTH0_ISSUER_URL}/.well-known/jwks.json`,
  });
  
  const key = await client.getSigningKey(kid);
  return key.getPublicKey();
};

async function verifyToken(req, res, next) {
  const token = req.headers.authorization?.split(' ')[1];
  
  if (!token) {
    return res.status(401).json({error: 'Missing token'});
  }
  
  try {
    // Decode header to get kid
    const decoded = jwt.decode(token, {complete: true});
    const kid = decoded.header.kid;
    
    // Get signing key
    const signingKey = await getSigningKey(kid);
    
    // Verify token
    const verified = jwt.verify(token, signingKey, {
      audience: process.env.AUTH0_AUDIENCE,
      issuer: process.env.AUTH0_ISSUER_URL,
    });
    
    // Attach to request
    req.user = verified;
    req.tenant_id = verified['org_id'];
    
    next();
  } catch (err) {
    res.status(401).json({error: 'Invalid token'});
  }
}

module.exports = {verifyToken};
```

```javascript
// routes/features.js
const {verifyToken} = require('../middleware/auth');

// Get user's features
app.get('/api/features', verifyToken, async (req, res) => {
  const {user, tenant_id} = req;
  
  // Check permission
  const roles = user['roles'] || [];
  if (!roles.includes('admin') && !roles.includes('engineer')) {
    return res.status(403).json({error: 'Insufficient permissions'});
  }
  
  // Get features for this tenant
  const features = await Feature.find({tenant_id});
  res.json(features);
});

// Generate new feature
app.post('/api/features/generate', verifyToken, async (req, res) => {
  const {user, tenant_id} = req;
  const {intent, tier} = req.body;
  
  // Check permission
  const roles = user['roles'] || [];
  if (!roles.includes('admin') && !roles.includes('engineer')) {
    return res.status(403).json({error: 'Insufficient permissions'});
  }
  
  // Check tier permission
  if (tier >= 4 && !roles.includes('admin')) {
    return res.status(403).json({error: 'Tier 4 generation requires admin role'});
  }
  
  // Generate feature (call LLM)
  const feature = await generateFeature({
    intent,
    tier,
    tenant_id,
    generated_by: user.sub,
  });
  
  res.json(feature);
});
```

---

## Data Flow & APIs

### Unified API Architecture

```
Client Request
  ├─ HTTP request with JWT in Authorization header
  ├─ Token contains: user_id, org_id, roles, permissions
  │
  ▼
API Gateway (request router)
  ├─ Extract JWT token
  ├─ Verify signature (check with Auth0's public key)
  ├─ Decode claims
  ├─ Route to appropriate service:
  │  ├─ RepairLink API
  │  ├─ EKOS API
  │  └─ VectorVolt API
  │
  ▼
Service Handler
  ├─ Extract user/org/roles from req.user
  ├─ Enforce tenant isolation:
  │  ├─ WHERE tenant_id = ?
  │  └─ WHERE org_id = ?
  ├─ Check RBAC:
  │  ├─ Does user have this role?
  │  └─ Does role have this permission?
  ├─ Execute business logic
  │  └─ (RepairLink, EKOS, or VectorVolt operation)
  │
  ▼
Database Query
  └─ All queries filtered by tenant_id
  
  ▼
Response to Client
  └─ JWT token still valid in header for next request
```

### Example Request Flow

**User generates new EKOS feature:**

```
1. User clicks "Generate" in EKOS UI
   POST /api/features/generate
   Authorization: Bearer eyJhbGc...
   {
     "intent": "Add filter by product line",
     "tier": 1
   }

2. API Gateway
   ├─ Extracts JWT
   ├─ Verifies signature with Auth0 public key
   ├─ Decodes: {
   │   "sub": "auth0|user123",
   │   "org_id": "customer_a",
   │   "roles": ["engineer"],
   │   "org_name": "Honda Dealer"
   │ }
   └─ Routes to EKOS service

3. EKOS Feature Generator
   ├─ Checks: user has 'engineer' role? YES ✓
   ├─ Checks: org_id matches request? YES ✓
   ├─ Calls LLM to generate feature code
   ├─ Stores feature in DB:
   │   {
   │     id: "feature_456",
   │     tenant_id: "customer_a",
   │     name: "ProductLineFilter",
   │     code: "...",
   │     created_by: "auth0|user123",
   │     created_at: "2026-07-12T10:30:00Z"
   │   }
   └─ Returns feature_456 to UI

4. EKOS UI
   ├─ Shows "Feature generated!"
   └─ Displays new feature in list
```

---

## Role-Based Access Control (RBAC)

### Permission Model

**Auth0 Roles → Permissions Matrix**

```
Role: admin
├─ EKOS:
│  ├─ generate:tier_1 (Widget generation)
│  ├─ generate:tier_2 (EKOS features)
│  ├─ generate:tier_3 (VectorVolt optimization)
│  ├─ generate:tier_4 (Core customization)
│  ├─ read:all_features
│  ├─ delete:any_feature
│  ├─ manage:users
│  └─ manage:roles
├─ RepairLink:
│  ├─ read:all_orders
│  ├─ write:all_orders
│  ├─ read:all_inventory
│  └─ write:all_inventory
└─ VectorVolt:
   ├─ read:all_data
   ├─ write:all_data
   └─ admin:database

Role: engineer
├─ EKOS:
│  ├─ generate:tier_1 ✓
│  ├─ generate:tier_2 ✓
│  ├─ generate:tier_3 ✓
│  ├─ generate:tier_4 ✗ (requires admin)
│  └─ read:own_features
├─ RepairLink:
│  ├─ read:all_orders ✓
│  └─ read:all_inventory ✓
└─ VectorVolt:
   └─ read:metadata_only

Role: manager
├─ EKOS:
│  ├─ generate:tier_1 ✓
│  └─ read:all_features ✓
├─ RepairLink:
│  ├─ read:all_orders ✓
│  ├─ write:own_orders ✓
│  ├─ read:all_inventory ✓
│  └─ approve:orders
└─ VectorVolt:
   └─ read:metadata_only

Role: technician
├─ EKOS:
│  └─ read:all_features ✓
├─ RepairLink:
│  ├─ read:all_orders ✓
│  ├─ write:own_orders ✓
│  ├─ read:all_inventory ✓
│  └─ update:order_status
└─ VectorVolt:
   └─ none

Role: viewer
├─ EKOS:
│  └─ read:all_features
├─ RepairLink:
│  ├─ read:all_orders
│  └─ read:all_inventory
└─ VectorVolt:
   └─ none
```

### RBAC Implementation

**In VectorVolt API:**

```rust
pub struct Permissions {
    ecos_generate_tier_1: bool,
    ecos_generate_tier_2: bool,
    ecos_generate_tier_3: bool,
    ecos_generate_tier_4: bool,
    repairlink_read_orders: bool,
    repairlink_write_orders: bool,
    vectorvolt_read_data: bool,
    vectorvolt_write_data: bool,
}

impl Permissions {
    pub fn from_roles(roles: &[String]) -> Self {
        let mut perms = Permissions {
            ecos_generate_tier_1: false,
            ecos_generate_tier_2: false,
            ecos_generate_tier_3: false,
            ecos_generate_tier_4: false,
            repairlink_read_orders: false,
            repairlink_write_orders: false,
            vectorvolt_read_data: false,
            vectorvolt_write_data: false,
        };
        
        for role in roles {
            match role.as_str() {
                "admin" => {
                    perms.ecos_generate_tier_1 = true;
                    perms.ecos_generate_tier_2 = true;
                    perms.ecos_generate_tier_3 = true;
                    perms.ecos_generate_tier_4 = true;
                    perms.repairlink_read_orders = true;
                    perms.repairlink_write_orders = true;
                    perms.vectorvolt_read_data = true;
                    perms.vectorvolt_write_data = true;
                }
                "engineer" => {
                    perms.ecos_generate_tier_1 = true;
                    perms.ecos_generate_tier_2 = true;
                    perms.ecos_generate_tier_3 = true;
                    perms.repairlink_read_orders = true;
                    perms.vectorvolt_read_data = true;
                }
                "manager" => {
                    perms.ecos_generate_tier_1 = true;
                    perms.repairlink_read_orders = true;
                    perms.repairlink_write_orders = true;
                    perms.vectorvolt_read_data = true;
                }
                "technician" => {
                    perms.repairlink_read_orders = true;
                    perms.repairlink_write_orders = true;
                }
                "viewer" => {
                    perms.repairlink_read_orders = true;
                }
                _ => {}
            }
        }
        
        perms
    }
}

// Usage in handlers
pub async fn generate_feature(
    user: AuthUser,
    tier: u32,
) -> Result<Feature> {
    let perms = Permissions::from_roles(&user.roles);
    
    match tier {
        1 if !perms.ecos_generate_tier_1 => {
            Err(Error::Forbidden("Cannot generate Tier 1 features"))
        }
        2 if !perms.ecos_generate_tier_2 => {
            Err(Error::Forbidden("Cannot generate Tier 2 features"))
        }
        3 if !perms.ecos_generate_tier_3 => {
            Err(Error::Forbidden("Cannot generate Tier 3 features"))
        }
        4 if !perms.ecos_generate_tier_4 => {
            Err(Error::Forbidden("Cannot generate Tier 4 features"))
        }
        _ => {
            // Generate feature
            Ok(feature)
        }
    }
}
```

---

## Security Model

### Token Security

**JWT Token Structure (from Auth0):**

```
Header:
{
  "alg": "RS256",
  "typ": "JWT",
  "kid": "key_id_from_jwks"
}

Payload:
{
  "iss": "https://vectorvolt.auth0.com/",
  "sub": "auth0|user123",
  "aud": "https://api.vectorvolt.io",
  "iat": 1689170000,
  "exp": 1689173600,
  "email": "john@customera.com",
  "org_id": "customer_a",
  "org_name": "Honda Dealer",
  "roles": ["engineer"],
  "permissions": ["ecos:generate:tier_1", "ecos:generate:tier_2"]
}

Signature:
HMACSHA256(
  base64UrlEncode(header) + "." +
  base64UrlEncode(payload),
  publicKey
)
```

**Token Validation (in API):**

```rust
pub async fn validate_token(token: &str) -> Result<TokenClaims> {
    // 1. Get Auth0's public key (JWKS)
    let jwks = fetch_jwks_from_auth0().await?;
    
    // 2. Decode without verification to get kid
    let unverified = jwt::decode_header::<Header>(token)?;
    let kid = unverified.kid.ok_or(Error::InvalidToken)?;
    
    // 3. Find the matching key
    let key = jwks.find_key(&kid)?;
    
    // 4. Verify signature
    let validation = Validation::new(Algorithm::RS256);
    let decoded = jwt::decode::<TokenClaims>(token, &key, &validation)?;
    
    // 5. Check expiry
    if decoded.claims.exp < current_unix_timestamp() {
        return Err(Error::TokenExpired);
    }
    
    // 6. Check audience (must match our API)
    if decoded.claims.aud != "https://api.vectorvolt.io" {
        return Err(Error::InvalidAudience);
    }
    
    Ok(decoded.claims)
}
```

### Multi-Tenancy Enforcement

```
For every query, enforce:
  1. User must have org_id in token
  2. Query must be filtered by tenant_id = user.org_id
  3. Results must only contain rows matching user's tenant_id

Example:
  // WRONG - returns all repair orders
  SELECT * FROM repair_orders WHERE status = 'pending'
  
  // RIGHT - returns only this tenant's orders
  SELECT * FROM repair_orders 
  WHERE status = 'pending' AND tenant_id = 'customer_a'

Enforcement points:
  - Database: ADD CHECK (tenant_id IS NOT NULL) to every table
  - ORM/Query Builder: Always add WHERE tenant_id = ? to queries
  - API Layer: Validate tenant_id from JWT matches request
```

---

## Implementation Roadmap

### Timeline: 12 Weeks

**Week 1-2: Auth0 Setup & RepairLink Integration**
```
Goals:
  - Auth0 tenant configured
  - RepairLink login via Auth0
  - Token validation middleware
  
Tasks:
  - Create Auth0 account
  - Configure applications (RepairLink, EKOS UI, VectorVolt API)
  - Implement login in RepairLink (Node.js)
  - Test login flow
  
Deliverable:
  - Users can log in with Auth0 credentials
  - Logout works
  - Session persists
```

**Week 3-4: EKOS UI Integration**
```
Goals:
  - EKOS UI login via Auth0
  - JWT tokens used for API calls
  
Tasks:
  - Install @auth0/auth0-react
  - Implement login/logout UI
  - Store tokens in localStorage
  - Add Authorization header to API calls
  - Test SPA flow
  
Deliverable:
  - Users can log in to EKOS UI
  - Tokens are sent with each API request
```

**Week 5-6: VectorVolt API Token Validation**
```
Goals:
  - VectorVolt API validates JWT tokens
  - User context available in all handlers
  
Tasks:
  - Implement JWT validation middleware
  - Extract user/org/roles from token
  - Add to request context
  - Test with various roles
  
Deliverable:
  - VectorVolt API rejects invalid tokens
  - Handlers have access to user info
```

**Week 7-8: Multi-Tenancy & RBAC**
```
Goals:
  - Each customer's data isolated
  - Roles control feature access
  
Tasks:
  - Create Auth0 organizations per customer
  - Add tenant_id to all DB tables
  - Enforce WHERE tenant_id = ? in queries
  - Implement role-based permission checks
  - Test cross-tenant access (should fail)
  
Deliverable:
  - Customer A cannot see Customer B's data
  - Roles control who can generate features
  - Audit logs track all access
```

**Week 9-10: Feature-Level Permissions**
```
Goals:
  - Only authorized users can generate features
  - Tier 4 generation restricted to admin
  
Tasks:
  - Add permission checks in feature generation
  - Test each role's access level
  - Implement rate limiting per org
  - Add feature audit trail
  
Deliverable:
  - Engineers can generate Tier 1-3 features
  - Only admins can generate Tier 4
  - All feature generation is logged
```

**Week 11-12: Integration Testing & Documentation**
```
Goals:
  - Full system integration verified
  - Runbook for operations team
  
Tasks:
  - End-to-end integration tests
  - Test multi-org scenarios
  - Create operational runbook
  - Document role/permission model
  - Security audit
  
Deliverable:
  - Comprehensive test coverage
  - Operations team trained
  - System ready for production
```

---

## Example Workflows

### Workflow 1: New Customer Onboarding

```
Scenario: Honda Dealer signs up for EKOS + RepairLink

1. Create Auth0 Organization
   curl -X POST https://vectorvolt.auth0.com/api/v2/organizations \
     -H "Authorization: Bearer $MGMT_API_TOKEN" \
     -d '{
       "name": "honda_dealer",
       "display_name": "Honda Dealer",
       "metadata": {
         "subscription_tier": "enterprise",
         "contract_start": "2026-07-12",
         "max_users": 50
       }
     }'

2. Invite Users to Organization
   curl -X POST https://vectorvolt.auth0.com/api/v2/organizations/honda_dealer/members \
     -H "Authorization: Bearer $MGMT_API_TOKEN" \
     -d '{
       "members": [
         {"email": "manager@honda.com"},
         {"email": "tech1@honda.com"},
         {"email": "tech2@honda.com"}
       ]
     }'

3. Assign Roles
   # Assign manager role to manager@honda.com
   curl -X PATCH https://vectorvolt.auth0.com/api/v2/users/auth0%7Cmanager123/roles \
     -H "Authorization: Bearer $MGMT_API_TOKEN" \
     -d '{
       "roles": ["manager"]
     }'

4. Create Tenant in VectorVolt
   POST /api/tenants {
     "name": "Honda Dealer",
     "auth0_org_id": "honda_dealer",
     "subscription_tier": "enterprise",
     "max_users": 50
   }

5. Sync Users to VectorVolt
   Auth0 rule runs automatically:
   - On each login, user record synced to VectorVolt
   - User roles synced from Auth0

6. User Logs In
   - Opens https://ekos.vectorvolt.io
   - Redirected to Auth0 login
   - Selects organization: "Honda Dealer"
   - Logs in with email
   - Redirected back to EKOS dashboard
   - Dashboard shows "Honda Dealer" organization
   - Can generate features, access RepairLink data

Result: New customer fully onboarded in <5 minutes
```

### Workflow 2: Feature Generation with Permissions

```
Scenario: Engineer at Honda generates new filter widget

1. Engineer logs in to EKOS UI
   - JWT token contains:
     org_id: "honda_dealer"
     roles: ["engineer"]

2. Engineer clicks "Generate Feature"
   - Enters: "Add filter by part number"
   - Tier: 1 (Widget)

3. Frontend makes API request
   POST /api/features/generate
   Authorization: Bearer eyJhbGc...
   {
     "intent": "Add filter by part number",
     "tier": 1
   }

4. Backend validates token
   - Signature valid? YES
   - Not expired? YES
   - Audience matches? YES
   - org_id present? YES ("honda_dealer")

5. Backend checks permissions
   - User roles: ["engineer"]
   - Required permission: "generate:tier_1"
   - Engineer role has tier_1? YES ✓

6. LLM generates feature code
   - Prompt includes:
     - Intent: "Add filter by part number"
     - Tenant: "honda_dealer"
     - User: "engineer@honda.com"
     - Tier: 1

7. Feature created in DB
   INSERT INTO features (
     id, tenant_id, name, code, tier, created_by, created_at
   ) VALUES (
     'feature_789',
     'honda_dealer',
     'PartNumberFilter',
     '...',
     1,
     'auth0|engineer123',
     NOW()
   )

8. Feature deployed
   - POD Manager creates new POD for feature
   - POD loaded with feature code
   - POD ready to serve requests

9. UI shows success
   - "Feature 'PartNumberFilter' generated!"
   - New feature appears in list
   - Can be used immediately

Result: New widget feature live in <5 minutes
```

### Workflow 3: Cross-Org Data Access Prevention

```
Scenario: Honda engineer tries to access Toyota data

1. Honda engineer logs in
   JWT contains: org_id: "honda_dealer"

2. Engineer makes request
   GET /api/repair-orders?filter=pending
   Authorization: Bearer honda_token

3. Backend retrieves orders
   SELECT * FROM repair_orders 
   WHERE status = 'pending' AND tenant_id = 'honda_dealer'
   
   Results: 5 Honda repair orders

4. Engineer somehow gets Toyota's JWT token
   (stolen, compromised, social engineering)
   JWT contains: org_id: "toyota_dealer"

5. Engineer makes request with Toyota token
   GET /api/repair-orders?filter=pending
   Authorization: Bearer toyota_token

6. Backend validates token
   - Signature: VALID (token signed by Auth0)
   - Expiry: NOT expired
   - Audience: VALID
   - org_id: "toyota_dealer"

7. Backend checks permissions
   - Roles in Toyota token: ["engineer"]
   - Permission check: YES, engineers can read orders

8. Backend enforces multi-tenancy
   SELECT * FROM repair_orders 
   WHERE status = 'pending' AND tenant_id = 'toyota_dealer'
   
   Results: 0 records (only Toyota's orders)
   OR: if engineer's org_id in token is different from request tenant_id, return 403

Result: Data is isolated by org_id in the database layer
Engineer cannot access Toyota data even with valid token
```

---

## Conclusion

Auth0 provides a robust, enterprise-grade identity layer for integrating RepairLink, EKOS, and VectorVolt into a unified system while maintaining:

✓ **Security**: Token-based auth with signature verification  
✓ **Multi-tenancy**: Org-level isolation at database and API layers  
✓ **Scalability**: Auth0 scales to millions of users  
✓ **Simplicity**: Standard OAuth2/OIDC, no custom auth to maintain  
✓ **Compliance**: SOC2, HIPAA, GDPR ready  

---

**END OF STRATEGY DOCUMENT**
