# 🚀 Cloudinary Setup - CRITICAL STEP

## ⚠️ MUST DO: Enable Unsigned Uploads (1 minute)

Before uploads work, you need to create an "Upload Preset" in Cloudinary:

### Step 1: Go to Cloudinary Settings
1. Login to: https://console.cloudinary.com/
2. Click **"Settings"** icon (⚙️) in top-right
3. Click **"Upload"** tab in left sidebar

### Step 2: Create Upload Preset
1. Scroll down to **"Upload presets"** section
2. Click **"Add upload preset"** link
3. Fill in:
   - **Upload preset name:** `esip_documents`
   - **Signing Mode:** Select **"Unsigned"**
   - **Folder:** `esip/documents`
   - Leave everything else as default
4. Click **"Save"** button

### Step 3: Copy Your Cloud Name
On the same page, at the top you'll see:
```
Cloud name: dxxxxx
```
**Copy this cloud name** and tell me!

---

## Why This Step?

Cloudinary requires "Upload Presets" for browser-based uploads (for security).
- **Signed uploads** = Need API secret (server-side only)
- **Unsigned uploads** = Safe for browser (with preset)

Once you create the preset, uploads will work instantly!

---

## After Setup

Tell me your **Cloud Name** and I'll update the code with it.

Example:
```
Cloud name: dmo4rbkmq
```

Then your uploads will work! 🎉
