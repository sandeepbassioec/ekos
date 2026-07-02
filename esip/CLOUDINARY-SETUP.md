# 📁 Cloudinary Free Storage Setup

## Why Cloudinary?
- ✅ **25GB FREE storage** (5x more than Firebase)
- ✅ 25GB/month bandwidth
- ✅ No credit card required
- ✅ Supports PDF, DOCX, images, videos

---

## Setup (5 minutes)

### Step 1: Create Cloudinary Account
1. Go to: https://cloudinary.com/users/register_free
2. Sign up with email (NO credit card needed)
3. After signup, you'll see your dashboard

### Step 2: Get Your Credentials
On dashboard, you'll see:
```
Cloud Name: dxxxxxxxx
API Key: 123456789012345
API Secret: abcdefghijklmnop-ABCD
```

**Copy these 3 values**

### Step 3: Add to Environment Variables
Create/edit `.env` file:
```bash
VITE_CLOUDINARY_CLOUD_NAME=your-cloud-name
VITE_CLOUDINARY_API_KEY=your-api-key
VITE_CLOUDINARY_API_SECRET=your-api-secret
```

### Step 4: Install Package
```bash
pnpm add cloudinary
```

---

## Code Changes

I'll update the upload code to use Cloudinary instead of Firebase Storage.

**Cloudinary uploads directly from browser** - no server needed!

---

## Limits (Free Tier)
- 25GB storage
- 25GB bandwidth/month
- 25 credits/month (transformations)
- Unlimited uploads

**Perfect for your use case!** Documents are small, so 25GB = thousands of files.

---

Ready to switch? Just say "switch to Cloudinary" and I'll update the code!
