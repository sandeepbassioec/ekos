# 🔥 FIRESTORE PERMISSION FIX - REQUIRED!

## Error You're Seeing:
```
FirebaseError: [code=permission-denied]: Missing or insufficient permissions.
```

## Root Cause:
New collections added (gaps, confusions, suggestions, securityRequirements, threats, nfrs, cloudGovernance) are blocked by Firestore security rules.

## Fix (5 minutes):

### Step 1: Open Firebase Console
Go to: https://console.firebase.google.com/

### Step 2: Select Your Project
Click on your project name

### Step 3: Navigate to Firestore Rules
- Click **Firestore Database** in left sidebar
- Click **Rules** tab at top

### Step 4: Replace Rules
Replace ALL existing rules with:

```
rules_version = '2';
service cloud.firestore {
  match /databases/{database}/documents {
    match /{document=**} {
      allow read, write: if request.auth != null;
    }
  }
}
```

### Step 5: Publish
Click **Publish** button

### Step 6: Test
- Refresh your app
- Try adding a Gap manually
- Should work now!

## What This Does:
- Allows any **authenticated user** to read/write to ALL collections
- Safe for development/prototyping
- For production, you'd add more specific rules per collection

## Collections Now Working:
✅ projects
✅ roadmaps
✅ roadmapItems
✅ documents
✅ meetings
✅ emails
✅ chats
✅ aiRequirements
✅ **gaps** (new)
✅ **confusions** (new)
✅ **suggestions** (new)
✅ **securityRequirements** (new)
✅ **threats** (new)
✅ **nfrs** (new)
✅ **cloudGovernance** (new)
