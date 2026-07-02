# 🔧 Fix: Firebase Storage Upload Blocked

## ❌ Issue
Document uploads are hanging/failing because **Firebase Storage security rules are blocking uploads**.

By default, Firebase Storage **denies all uploads** for security.

---

## ✅ Solution: Update Storage Rules (2 minutes)

### **Step 1: Open Firebase Storage Rules**

Go to this URL:
```
https://console.firebase.google.com/project/esip-d6104/storage/rules
```

Or:
1. Go to Firebase Console: https://console.firebase.google.com/project/esip-d6104
2. Click **"Storage"** in left sidebar
3. Click **"Rules"** tab

### **Step 2: Replace Existing Rules**

You'll see something like:
```
rules_version = '2';
service firebase.storage {
  match /b/{bucket}/o {
    match /{allPaths=**} {
      allow read, write: if false; // ❌ This blocks everything
    }
  }
}
```

**Replace with:**
```
rules_version = '2';
service firebase.storage {
  match /b/{bucket}/o {
    // Allow authenticated users to upload/read documents
    match /documents/{roadmapItemId}/{fileName} {
      allow read: if request.auth != null;
      allow write: if request.auth != null;
      allow delete: if request.auth != null;
    }
    
    // Deny everything else
    match /{allPaths=**} {
      allow read, write: if false;
    }
  }
}
```

### **Step 3: Publish Rules**

1. Click **"Publish"** button (top right)
2. Confirm by clicking **"Publish"** again

**Status should show:** ✅ "Rules published successfully"

---

## 🧪 Test Upload Again

1. **Refresh your browser** (Ctrl+Shift+R or Cmd+Shift+R)
2. Go to **Gantt Chart** → Click a roadmap item
3. Click **"Upload Document"**
4. Select file and fill form
5. Click **"Upload"**

**Expected:**
- ✅ Console logs: "File uploaded to storage successfully"
- ✅ Toast: "Document uploaded successfully"
- ✅ File appears in Documents tab

---

## 🔍 Verify Upload in Firebase Console

After successful upload, check:

**Firestore Database:**
```
https://console.firebase.google.com/project/esip-d6104/firestore/data
```
- Go to `documents` collection
- You should see your document metadata

**Firebase Storage:**
```
https://console.firebase.google.com/project/esip-d6104/storage/files
```
- Go to `documents/` folder
- You should see your uploaded file

---

## 📋 What the Rules Do

```javascript
allow read: if request.auth != null;   // Only logged-in users can download
allow write: if request.auth != null;  // Only logged-in users can upload
allow delete: if request.auth != null; // Only logged-in users can delete
```

**Security:**
- ✅ Anonymous users **cannot** upload/download
- ✅ Only authenticated users can access
- ✅ Files organized by roadmapItemId
- ✅ No size limits (uses Firebase default 5GB)

---

## 🐛 Still Having Issues?

### **Error: "storage/unauthorized"**
**Fix:** You published the rules - wait 10 seconds and try again

### **Error: "storage/object-not-found"**
**Cause:** Different error - file path issue
**Fix:** Check console logs for actual path being used

### **No error, just infinite "Uploading..."**
**Fix:** 
1. Open browser console (F12)
2. Look for red errors
3. Share the error message

### **File uploads but doesn't appear in list**
**Fix:** 
1. Close and reopen the Requirements Panel
2. Refresh the page
3. Check Firestore Console to confirm document was created

---

## 🎯 Quick Summary

1. ✅ Go to: https://console.firebase.google.com/project/esip-d6104/storage/rules
2. ✅ Replace rules with the code above
3. ✅ Click "Publish"
4. ✅ Refresh browser and try upload again

**That's it! Your uploads will work after this.**

---

## 💡 Advanced: Production-Ready Rules (Optional)

For production, add file size limits and type restrictions:

```javascript
rules_version = '2';
service firebase.storage {
  match /b/{bucket}/o {
    match /documents/{roadmapItemId}/{fileName} {
      allow read: if request.auth != null;
      
      // Upload: max 10MB, only PDF/DOCX/TXT
      allow write: if request.auth != null 
                   && request.resource.size < 10 * 1024 * 1024
                   && request.resource.contentType.matches('application/pdf|application/.*word.*|text/.*');
      
      allow delete: if request.auth != null;
    }
  }
}
```

This adds:
- 📏 Max file size: 10MB
- 📄 Allowed types: PDF, Word documents, text files
