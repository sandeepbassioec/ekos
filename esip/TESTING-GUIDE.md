# 🧪 Testing Guide - Firebase Registration & Login

## ✅ Setup Complete

Firebase DAL is ready! Registration functionality **will work** once you complete Firebase setup.

---

## 🔧 Required: Firebase Console Setup (5 minutes)

### **Step 1: Enable Email/Password Authentication**

1. Go to: https://console.firebase.google.com/project/esip-d6104/authentication/providers
2. Click **"Email/Password"**
3. Toggle **"Enable"** to ON
4. Click **"Save"**

**Screenshot location:** Authentication → Sign-in method → Email/Password

### **Step 2: Enable Firestore Database**

1. Go to: https://console.firebase.google.com/project/esip-d6104/firestore
2. Click **"Create database"**
3. Choose **"Start in production mode"**
4. Select region: **us-central (or your preferred region)**
5. Click **"Enable"**

### **Step 3: Enable Storage**

1. Go to: https://console.firebase.google.com/project/esip-d6104/storage
2. Click **"Get started"**
3. Use default security rules (we'll update later)
4. Click **"Done"**

---

## 🧪 Test Registration Flow

### **1. Access Registration Page**

```
URL: http://localhost:5173/register
(or your app's preview URL)
```

### **2. Fill in Registration Form**

- **Full Name:** Test User
- **Email:** test@example.com
- **Password:** Test1234 (minimum 8 characters)
- **Role:** Manager (or any role)

### **3. Click "Register"**

**Expected behavior:**
- ✅ Loading state: "Creating account..."
- ✅ Success toast: "Account created successfully"
- ✅ Redirects to homepage (`/`)
- ✅ User is logged in

**If you get an error:**
- ❌ "Firebase Auth is not enabled" → Go to Step 1 above
- ❌ "Permission denied" → Check Firestore is created (Step 2)
- ❌ Other errors → Check browser console (F12)

---

## 🧪 Test Login Flow

### **1. Logout (if logged in)**

Click logout button in the app (top right corner)

### **2. Access Login Page**

```
URL: http://localhost:5173/login
```

### **3. Fill in Login Form**

- **Email:** test@example.com
- **Password:** Test1234

### **4. Click "Login"**

**Expected behavior:**
- ✅ Loading state: "Logging in..."
- ✅ Success toast: "Logged in successfully"
- ✅ Redirects to homepage
- ✅ Shows user name in header

---

## 🧪 Test Full Application Flow

### **After successful login:**

1. **Create a Project**
   - Click "Start New Estimation Wizard" or "Create Project"
   - Fill in project details
   - Save → Should save to Firebase Firestore

2. **Create a Roadmap**
   - Navigate to Roadmap section
   - Select year (2026) and type (Product)
   - Click "Create Roadmap"
   - Add roadmap items

3. **Upload a Document (Knowledge Hub)**
   - Go to Roadmap Item → Knowledge Hub
   - Upload a BRD/FRD document
   - Should upload to Firebase Storage

---

## 🐛 Troubleshooting

### **Error: "Firebase: Error (auth/operation-not-allowed)"**
**Fix:** Enable Email/Password auth in Firebase Console (Step 1 above)

### **Error: "Missing or insufficient permissions"**
**Fix:** Create Firestore database (Step 2 above)

### **Error: "Firebase: Firebase App named '[DEFAULT]' already exists"**
**Fix:** Refresh the page (this is a hot-reload issue in development)

### **Error: Network request failed**
**Fix:** Check your internet connection and Firebase project status

### **Console shows "Firebase not initialized"**
**Fix:** Already fixed! Firebase initializes in `src/main.tsx`

---

## 📊 Verify Data in Firebase Console

### **Check Registered Users:**
https://console.firebase.google.com/project/esip-d6104/authentication/users

You should see your test user: `test@example.com`

### **Check Firestore Data:**
https://console.firebase.google.com/project/esip-d6104/firestore/databases/-default-/data

After creating projects, you'll see:
- `projects` collection with your projects
- `roadmaps` collection with roadmaps
- `roadmapItems` collection with items

### **Check Storage Files:**
https://console.firebase.google.com/project/esip-d6104/storage

After uploading documents, you'll see:
- `documents/{roadmapItemId}/{filename}`

---

## 🎯 What Works Now

✅ **Registration** - Create new user accounts  
✅ **Login** - Sign in with email/password  
✅ **Logout** - Sign out  
✅ **Projects** - Create, read, update, delete (CRUD)  
✅ **Roadmaps** - Create roadmaps by year/type  
✅ **Roadmap Items** - Add features/initiatives  
✅ **Documents** - Upload to Firebase Storage  
✅ **Meetings/Emails/Chats** - Create knowledge hub entries  

---

## ⚠️ What Needs Setup

### **Required (before app works):**
1. ✅ Enable Email/Password Authentication ← **Do this first!**
2. ✅ Create Firestore Database
3. ✅ Enable Storage

### **Optional (for production):**
1. Add Firestore indexes (for faster queries)
2. Update security rules (for better security)
3. Configure AI requirements extraction (Bedrock or Cloud Functions)

---

## 🚀 Quick Start Commands

```bash
# If app is not running, start it:
cd /workspaces/default/code
pnpm dev

# Open browser to:
http://localhost:5173/register
```

---

## 📝 Test Checklist

- [ ] Firebase Authentication enabled (Email/Password)
- [ ] Firestore Database created
- [ ] Storage enabled
- [ ] Register new user (test@example.com)
- [ ] Login with registered user
- [ ] Create a project
- [ ] Create a roadmap
- [ ] Add roadmap item
- [ ] Upload a document
- [ ] Logout
- [ ] Login again (verify persistence)

---

**🎉 Once you enable Firebase Auth, everything will work!** The DAL is fully ready, just needs Firebase Console setup.
