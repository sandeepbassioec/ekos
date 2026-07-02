/**
 * Firebase Configuration
 */

import { initializeApp, getApps, type FirebaseApp } from 'firebase/app';
import { getAuth, type Auth } from 'firebase/auth';
import { getFirestore, initializeFirestore, type Firestore } from 'firebase/firestore';
import { getStorage, type FirebaseStorage } from 'firebase/storage';

const firebaseConfig = {
  apiKey: "AIzaSyAV-ZDLtWwYOUd1v2h0xBYL69QGs1iLt2U",
  authDomain: "esip-d6104.firebaseapp.com",
  projectId: "esip-d6104",
  storageBucket: "esip-d6104.firebasestorage.app",
  messagingSenderId: "406226053141",
  appId: "1:406226053141:web:670f96466e60d4c63dc61c",
  measurementId: "G-6JE3W84XQ3"
};

let app: FirebaseApp;
let auth: Auth;
let db: Firestore;
let storage: FirebaseStorage;

export function initFirebase() {
  if (getApps().length === 0) {
    app = initializeApp(firebaseConfig);
  } else {
    app = getApps()[0];
  }

  auth = getAuth(app);
  // Initialize Firestore with ignoreUndefinedProperties so writes never throw on
  // undefined fields (common in AI results and partial wizard data). Fall back to
  // getFirestore if Firestore was already started (e.g. during Vite HMR).
  try {
    db = initializeFirestore(app, { ignoreUndefinedProperties: true });
  } catch {
    db = getFirestore(app);
  }
  storage = getStorage(app);

  return { app, auth, db, storage };
}

export function getFirebaseAuth() {
  if (!auth) initFirebase();
  return auth;
}

export function getFirebaseFirestore() {
  if (!db) initFirebase();
  return db;
}

export function getFirebaseStorage() {
  if (!storage) initFirebase();
  return storage;
}
