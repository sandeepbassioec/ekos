# 🔧 Fix: Firebase Auth Configuration Not Found

## Error
```
Firebase error (auth/configuration-not-found)
```

This means Firebase Authentication hasn't been set up in your Firebase Console.

---

## ✅ Solution: Enable Authentication (Step-by-Step)

### **Step 1: Go to Firebase Console**

Open this URL in your browser:
```
https://console.firebase.google.com/project/esip-d6104/authentication
```

### **Step 2: Get Started with Authentication**

You should see one of these screens:

**Option A: If you see "Get Started" button**
- Click **"Get Started"** button
- This will initialize Authentication for your project

**Option B: If you see "Sign-in method" tab**
- You're already in Authentication
- Go to Step 3

### **Step 3: Enable Email/Password Provider**

1. Click on the **"Sign-in method"** tab (top of page)
2. In the "Sign-in providers" list, find **"Email/Password"**
3. Click on **"Email/Password"** row
4. Toggle **"Enable"** switch to ON
5. Click **"Save"**

**Screenshot confirmation:** You should see "Email/Password" with status "Enabled"

---

## 🧪 Verify Setup

### **Check Authentication is Enabled:**

1. Go to: https://console.firebase.google.com/project/esip-d6104/authentication/users
2. You should see an empty users table (no error)
3. This confirms Authentication is working

---

## 🔄 Test Registration Again

1. **Refresh your browser** (hard refresh: Ctrl+Shift+R or Cmd+Shift+R)
2. Go to: `http://localhost:5173/register`
3. Fill in the form:
   - Email: `test@example.com`
   - Password: `Test1234`
   - Full Name: `Test User`
4. Click **"Register"**

**Expected:** ✅ Account created successfully

---

## 🐛 If Still Getting Error

### **Double-check these:**

1. **Project ID is correct:**
   - Go to: https://console.firebase.google.com/project/esip-d6104/settings/general
   - Verify "Project ID" = `esip-d6104`

2. **Get fresh config (if project ID is different):**

   If your project ID is NOT `esip-d6104`, get the correct config:

   a. Go to: https://console.firebase.google.com/project/YOUR_PROJECT_ID/settings/general
   b. Scroll to "Your apps" section
   c. Click the **Web app icon** (</>) or "Add app"
   d. Register app (name: "ESIP Web App")
   e. Copy the `firebaseConfig` object
   f. Replace in: `/src/app/lib/db/firebase-config.ts`

---

## 📋 Firebase Console Checklist

Go through these URLs and verify each is set up:

- [ ] **Authentication enabled:**
      https://console.firebase.google.com/project/esip-d6104/authentication

- [ ] **Email/Password provider enabled:**
      https://console.firebase.google.com/project/esip-d6104/authentication/providers

- [ ] **Firestore created:**
      https://console.firebase.google.com/project/esip-d6104/firestore

- [ ] **Storage enabled:**
      https://console.firebase.google.com/project/esip-d6104/storage

---

## 🎯 Common Causes

### **Cause 1: Authentication not initialized**
**Fix:** Click "Get Started" in Authentication section

### **Cause 2: Wrong Firebase project**
**Fix:** Verify project ID in Firebase Console matches config

### **Cause 3: Web app not registered**
**Fix:** Register a web app in Project Settings → General → Your apps

### **Cause 4: Browser cache**
**Fix:** Hard refresh (Ctrl+Shift+R) or clear cache

---

## 🔍 Alternative: Create New Firebase Project

If the project `esip-d6104` has issues, create a fresh one:

### **1. Create New Project:**
https://console.firebase.google.com/

- Click "Add project"
- Name: "ESIP Platform"
- Follow the wizard

### **2. Get Config:**

- Go to Project Settings → General
- Scroll to "Your apps"
- Click Web icon (</>) 
- Register app: "ESIP Web"
- Copy `firebaseConfig`

### **3. Update Code:**

Edit `/src/app/lib/db/firebase-config.ts`:

```typescript
const firebaseConfig = {
  // Paste your new config here
  apiKey: "YOUR_NEW_API_KEY",
  authDomain: "your-project.firebaseapp.com",
  projectId: "your-project-id",
  storageBucket: "your-project.appspot.com",
  messagingSenderId: "123456789",
  appId: "1:123456789:web:abcdef",
  measurementId: "G-ABCDEFGH"
};
```

### **4. Enable Services:**

- Authentication → Get Started → Enable Email/Password
- Firestore → Create Database (production mode)
- Storage → Get Started

---

## ✅ Success Indicators

After fixing, you should see:

1. ✅ **Authentication page loads** without errors
2. ✅ **Registration creates user** in Firebase Console
3. ✅ **Login works** with created user
4. ✅ **Firestore collections** appear when you create projects

---

**Most likely fix:** Just click "Get Started" in the Authentication section of Firebase Console!
