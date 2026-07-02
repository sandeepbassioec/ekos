/**
 * Firebase Implementation of Database Abstraction Layer
 */

import {
  createUserWithEmailAndPassword,
  signInWithEmailAndPassword,
  signOut as firebaseSignOut,
  updateProfile,
  type User as FirebaseUser,
} from 'firebase/auth';

import {
  collection,
  doc,
  getDoc,
  getDocs,
  addDoc,
  setDoc,
  updateDoc,
  deleteDoc,
  query,
  where,
  orderBy,
  serverTimestamp,
  Timestamp,
  type QueryConstraint,
} from 'firebase/firestore';

import {
  ref,
  uploadBytes,
  getDownloadURL,
  deleteObject,
} from 'firebase/storage';

import { getFirebaseAuth, getFirebaseFirestore, getFirebaseStorage } from './firebase-config';
import { uploadToCloudinary, getCloudinaryUrl } from '../cloudinary';
import type { IDatabase, AuthProvider } from './interface';
import type {
  AuthUser,
  AuthSession,
  Project,
  Roadmap,
  RoadmapItem,
  Document,
  Meeting,
  Email,
  Chat,
  AIRequirement,
  Gap,
  Confusion,
  Suggestion,
  SecurityRequirement,
  Threat,
  NonFunctionalRequirement,
  CloudGovernance,
  FormalRequirement,
  Conflict,
  SharedContentView,
  PI,
  Sprint,
  Requirement,
  Story,
  WorkflowRequest,
  TradeoffAnalysis,
  UserProfile,
  Invite,
  ProjectMember,
  ProjectRole,
  DesignerDocument,
} from './types';

// Helpers
function timestampToISO(timestamp: any): string {
  if (timestamp instanceof Timestamp) return timestamp.toDate().toISOString();
  if (timestamp?.toDate) return timestamp.toDate().toISOString();
  if (typeof timestamp === 'string') return timestamp;
  return new Date().toISOString();
}

const USERS_COLLECTION = 'users';
const DEFAULT_ROLE = 'manager'; // fallback for legacy users that have no profile doc yet
const DEFAULT_LICENSE = 'contributor'; // legacy/self-registered users keep full authoring
const DEFAULT_STATUS = 'active';

interface ProfileFields {
  role?: string;
  licenseTier?: string;
  status?: string;
}

function toAuthUser(firebaseUser: FirebaseUser, profile?: ProfileFields): AuthUser {
  return {
    id: firebaseUser.uid,
    email: firebaseUser.email!,
    fullName: firebaseUser.displayName || firebaseUser.email!,
    role: profile?.role || DEFAULT_ROLE,
    licenseTier: profile?.licenseTier || DEFAULT_LICENSE,
    status: profile?.status || DEFAULT_STATUS,
  };
}

/** Read the user's authorization fields from their Firestore profile doc (`users/{uid}`). */
async function fetchUserProfile(uid: string): Promise<ProfileFields | undefined> {
  try {
    const db = getFirebaseFirestore();
    const snap = await getDoc(doc(db, USERS_COLLECTION, uid));
    if (!snap.exists()) return undefined;
    const d = snap.data();
    return { role: d.role, licenseTier: d.licenseTier, status: d.status };
  } catch (error) {
    console.warn('Could not read user profile:', error);
    return undefined;
  }
}

/** Create or merge the user's Firestore profile doc. */
async function writeUserProfile(
  uid: string,
  data: { email: string; fullName: string; role: string; licenseTier?: string; status?: string; defaultPersonaId?: string; invitedBy?: string },
): Promise<void> {
  const db = getFirebaseFirestore();
  await setDoc(
    doc(db, USERS_COLLECTION, uid),
    {
      email: data.email,
      fullName: data.fullName,
      role: data.role,
      licenseTier: data.licenseTier || DEFAULT_LICENSE,
      status: data.status || DEFAULT_STATUS,
      ...(data.defaultPersonaId ? { defaultPersonaId: data.defaultPersonaId } : {}),
      ...(data.invitedBy ? { invitedBy: data.invitedBy } : {}),
      updatedAt: serverTimestamp(),
    },
    { merge: true },
  );
}

/** Return the stored profile; if a user has none yet (legacy), create one with defaults. */
async function ensureUserProfile(firebaseUser: FirebaseUser): Promise<ProfileFields> {
  const existing = await fetchUserProfile(firebaseUser.uid);
  if (existing?.role) {
    return { role: existing.role, licenseTier: existing.licenseTier || DEFAULT_LICENSE, status: existing.status || DEFAULT_STATUS };
  }
  await writeUserProfile(firebaseUser.uid, {
    email: firebaseUser.email || '',
    fullName: firebaseUser.displayName || firebaseUser.email || '',
    role: DEFAULT_ROLE,
  });
  return { role: DEFAULT_ROLE, licenseTier: DEFAULT_LICENSE, status: DEFAULT_STATUS };
}

async function getAll<T>(collectionName: string, constraints: QueryConstraint[] = []): Promise<T[]> {
  const db = getFirebaseFirestore();
  const q = query(collection(db, collectionName), ...constraints);
  const snapshot = await getDocs(q);
  return snapshot.docs.map((doc) => ({
    id: doc.id,
    ...doc.data(),
    createdAt: timestampToISO(doc.data().createdAt),
    updatedAt: timestampToISO(doc.data().updatedAt),
  })) as T[];
}

async function getById<T>(collectionName: string, id: string): Promise<T> {
  const db = getFirebaseFirestore();
  const docRef = doc(db, collectionName, id);
  const docSnap = await getDoc(docRef);
  if (!docSnap.exists()) throw new Error(`${collectionName} not found`);
  return {
    id: docSnap.id,
    ...docSnap.data(),
    createdAt: timestampToISO(docSnap.data()?.createdAt),
    updatedAt: timestampToISO(docSnap.data()?.updatedAt),
  } as T;
}

async function create<T>(collectionName: string, data: any): Promise<T> {
  const db = getFirebaseFirestore();
  const docRef = await addDoc(collection(db, collectionName), {
    ...data,
    createdAt: serverTimestamp(),
    updatedAt: serverTimestamp(),
  });
  return getById<T>(collectionName, docRef.id);
}

async function update<T>(collectionName: string, id: string, updates: any): Promise<T> {
  const db = getFirebaseFirestore();
  const docRef = doc(db, collectionName, id);
  await updateDoc(docRef, { ...updates, updatedAt: serverTimestamp() });
  return getById<T>(collectionName, id);
}

async function deleteDoc_(collectionName: string, id: string): Promise<void> {
  const db = getFirebaseFirestore();
  await deleteDoc(doc(db, collectionName, id));
}

// Auth Provider
const authProvider: AuthProvider = {
  async signUp(email, password, fullName, role = DEFAULT_ROLE, extras): Promise<AuthSession> {
    const auth = getFirebaseAuth();
    const credential = await createUserWithEmailAndPassword(auth, email, password);
    await updateProfile(credential.user, { displayName: fullName });
    // Persist role + license in a Firestore profile doc. Firebase Auth can't hold custom
    // claims client-side (those need the Admin SDK), so the profile doc is the source of
    // truth for authorization. `extras` carries invite-assigned license/persona (Epic G).
    await writeUserProfile(credential.user.uid, {
      email,
      fullName,
      role,
      licenseTier: extras?.licenseTier,
      status: extras?.status,
      defaultPersonaId: extras?.defaultPersonaId,
      invitedBy: extras?.invitedBy,
    });
    const token = await credential.user.getIdToken();
    return {
      access_token: token,
      user: toAuthUser(credential.user, { role, licenseTier: extras?.licenseTier, status: extras?.status }),
    };
  },

  async signIn(email: string, password: string): Promise<AuthSession> {
    const auth = getFirebaseAuth();
    const credential = await signInWithEmailAndPassword(auth, email, password);
    const profile = await ensureUserProfile(credential.user); // reads profile; self-heals legacy users
    const token = await credential.user.getIdToken();
    return { access_token: token, user: toAuthUser(credential.user, profile) };
  },

  async signOut(): Promise<void> {
    await firebaseSignOut(getFirebaseAuth());
  },

  async getSession(): Promise<AuthSession | null> {
    const auth = getFirebaseAuth();
    const user = auth.currentUser;
    if (!user) return null;
    const profile = await fetchUserProfile(user.uid);
    const token = await user.getIdToken();
    return { access_token: token, user: toAuthUser(user, profile) };
  },

  async getUser(): Promise<AuthUser | null> {
    const user = getFirebaseAuth().currentUser;
    if (!user) return null;
    const profile = await fetchUserProfile(user.uid);
    return toAuthUser(user, profile);
  },
};

// Database Implementation
export const firebaseDatabase: IDatabase = {
  auth: authProvider,

  projects: {
    async getAll(userId: string): Promise<Project[]> {
      return getAll<Project>('projects', [
        where('userId', '==', userId),
        // Temporarily removed orderBy to avoid composite index requirement
        // orderBy('updatedAt', 'desc'),
      ]);
    },
    async getById(id: string, userId: string): Promise<Project> {
      const project = await getById<Project>('projects', id);
      if (project.userId !== userId) throw new Error('Unauthorized');
      return project;
    },
    async create(data: Omit<Project, 'id' | 'createdAt' | 'updatedAt'>): Promise<Project> {
      return create<Project>('projects', data);
    },
    async update(id: string, userId: string, updates: Partial<Project>): Promise<Project> {
      const project = await getById<Project>('projects', id);
      if (project.userId !== userId) throw new Error('Unauthorized');
      return update<Project>('projects', id, updates);
    },
    async delete(id: string, userId: string): Promise<void> {
      const project = await getById<Project>('projects', id);
      if (project.userId !== userId) throw new Error('Unauthorized');

      // CASCADE DELETE: Delete all related data
      console.log(`Cascade deleting project ${id} and all related data...`);

      // 1. Get all roadmaps for this project
      const roadmaps = await getAll<Roadmap>('roadmaps', [where('projectId', '==', id)]);
      console.log(`Found ${roadmaps.length} roadmaps to delete`);

      // 2. For each roadmap, delete all roadmap items and their knowledge hub data
      for (const roadmap of roadmaps) {
        const roadmapItems = await getAll<RoadmapItem>('roadmapItems', [
          where('roadmapId', '==', roadmap.id),
        ]);
        console.log(`Found ${roadmapItems.length} roadmap items in roadmap ${roadmap.id}`);

        // 3. For each roadmap item, delete knowledge hub data
        for (const item of roadmapItems) {
          // Delete documents
          const documents = await getAll<Document>('documents', [
            where('roadmapItemId', '==', item.id),
          ]);
          for (const doc of documents) {
            await deleteDoc_('documents', doc.id);
          }

          // Delete meetings
          const meetings = await getAll<Meeting>('meetings', [
            where('roadmapItemId', '==', item.id),
          ]);
          for (const meeting of meetings) {
            await deleteDoc_('meetings', meeting.id);
          }

          // Delete emails
          const emails = await getAll<Email>('emails', [
            where('roadmapItemId', '==', item.id),
          ]);
          for (const email of emails) {
            await deleteDoc_('emails', email.id);
          }

          // Delete chats
          const chats = await getAll<Chat>('chats', [
            where('roadmapItemId', '==', item.id),
          ]);
          for (const chat of chats) {
            await deleteDoc_('chats', chat.id);
          }

          // Delete AI requirements
          const aiReqs = await getAll<AIRequirement>('aiRequirements', [
            where('roadmapItemId', '==', item.id),
          ]);
          for (const aiReq of aiReqs) {
            await deleteDoc_('aiRequirements', aiReq.id);
          }

          console.log(`Deleted knowledge hub data for roadmap item ${item.id}`);
        }

        // 4. Delete all roadmap items
        for (const item of roadmapItems) {
          await deleteDoc_('roadmapItems', item.id);
        }
        console.log(`Deleted ${roadmapItems.length} roadmap items from roadmap ${roadmap.id}`);

        // 5. Delete the roadmap
        await deleteDoc_('roadmaps', roadmap.id);
        console.log(`Deleted roadmap ${roadmap.id}`);
      }

      // 6. Finally delete the project
      await deleteDoc_('projects', id);
      console.log(`Cascade delete complete for project ${id}`);
    },
  },

  roadmaps: {
    async getById(id: string): Promise<Roadmap> {
      return getById<Roadmap>('roadmaps', id);
    },
    async getByProject(projectId: string): Promise<Roadmap[]> {
      return getAll<Roadmap>('roadmaps', [
        where('projectId', '==', projectId),
        // Temporarily removed orderBy to avoid composite index requirement
        // orderBy('year', 'desc'),
      ]);
    },
    async getByYearAndType(projectId: string, year: number, type: string): Promise<Roadmap | null> {
      const results = await getAll<Roadmap>('roadmaps', [
        where('projectId', '==', projectId),
        where('year', '==', year),
        where('type', '==', type),
      ]);
      return results[0] || null;
    },
    async create(data: Omit<Roadmap, 'id' | 'createdAt' | 'updatedAt'>): Promise<Roadmap> {
      return create<Roadmap>('roadmaps', data);
    },
    async delete(id: string): Promise<void> {
      await deleteDoc_('roadmaps', id);
    },
  },

  roadmapItems: {
    async getById(id: string): Promise<RoadmapItem> {
      return getById<RoadmapItem>('roadmapItems', id);
    },
    async getByRoadmap(roadmapId: string): Promise<RoadmapItem[]> {
      return getAll<RoadmapItem>('roadmapItems', [
        where('roadmapId', '==', roadmapId),
        // Temporarily removed orderBy to avoid composite index requirement
        // orderBy('startDate', 'asc'),
      ]);
    },
    async create(data: Omit<RoadmapItem, 'id' | 'createdAt' | 'updatedAt'>): Promise<RoadmapItem> {
      return create<RoadmapItem>('roadmapItems', data);
    },
    async update(id: string, updates: Partial<RoadmapItem>): Promise<RoadmapItem> {
      return update<RoadmapItem>('roadmapItems', id, updates);
    },
    async delete(id: string): Promise<void> {
      await deleteDoc_('roadmapItems', id);
    },
  },

  documents: {
    async getByRoadmapItem(roadmapItemId: string): Promise<Document[]> {
      const db = getFirebaseFirestore();
      const q = query(
        collection(db, 'documents'),
        where('roadmapItemId', '==', roadmapItemId)
        // Temporarily removed orderBy to avoid composite index requirement
        // orderBy('uploadedAt', 'desc')
      );
      const snapshot = await getDocs(q);
      return snapshot.docs.map((doc) => ({
        id: doc.id,
        ...doc.data(),
        uploadedAt: timestampToISO(doc.data().uploadedAt),
      })) as Document[];
    },
    async create(docData: Omit<Document, 'id' | 'uploadedAt'>, file: File): Promise<Document> {
      try {
        console.log('Starting document upload to Cloudinary...', { file: file.name, size: file.size });

        // Upload to Cloudinary
        const cloudinaryResult = await uploadToCloudinary(file, `esip/roadmaps/${docData.roadmapItemId}`);
        console.log('File uploaded to Cloudinary successfully:', cloudinaryResult.secureUrl);

        // Save metadata to Firestore
        const db = getFirebaseFirestore();
        console.log('Creating Firestore document metadata...');
        const docRef = await addDoc(collection(db, 'documents'), {
          ...docData,
          storagePath: cloudinaryResult.publicId, // Store Cloudinary public ID
          fileUrl: cloudinaryResult.secureUrl,    // Direct download URL
          fileSize: cloudinaryResult.bytes,
          uploadedAt: serverTimestamp(),
        });
        console.log('Firestore document created:', docRef.id);

        const docSnap = await getDoc(docRef);
        const result = {
          id: docRef.id,
          ...docSnap.data(),
          uploadedAt: timestampToISO(docSnap.data()?.uploadedAt),
        } as Document;

        console.log('Document upload complete:', result);
        return result;
      } catch (error) {
        console.error('Error in document create:', error);
        throw error;
      }
    },
    async update(id: string, updates: Pick<Document, 'name' | 'type' | 'version'>): Promise<Document> {
      return update<Document>('documents', id, updates);
    },
    async delete(id: string): Promise<void> {
      const document = await getById<Document>('documents', id);
      // Note: Cloudinary delete requires server-side API with secret
      // For now, just delete from Firestore (files remain in Cloudinary free tier)
      // TODO: Implement server-side Cloudinary delete endpoint if needed
      console.log('Deleting document metadata from Firestore (Cloudinary file remains):', id);
      await deleteDoc_('documents', id);
    },
    async getDownloadUrl(id: string): Promise<string> {
      const document = await getById<Document>('documents', id);
      // Return the stored Cloudinary URL directly
      return (document as any).fileUrl || getCloudinaryUrl(document.storagePath);
    },
  },

  meetings: {
    async getByRoadmapItem(roadmapItemId: string): Promise<Meeting[]> {
      const db = getFirebaseFirestore();
      const q = query(
        collection(db, 'meetings'),
        where('roadmapItemId', '==', roadmapItemId)
        // Temporarily removed orderBy to avoid composite index requirement
        // orderBy('date', 'desc')
      );
      const snapshot = await getDocs(q);
      return snapshot.docs.map((doc) => ({
        id: doc.id,
        ...doc.data(),
        createdAt: timestampToISO(doc.data().createdAt),
      })) as Meeting[];
    },
    async create(data: Omit<Meeting, 'id' | 'createdAt'>): Promise<Meeting> {
      return create<Meeting>('meetings', data);
    },
    async update(id: string, updates: Partial<Meeting>): Promise<Meeting> {
      return update<Meeting>('meetings', id, updates);
    },
    async delete(id: string): Promise<void> {
      await deleteDoc_('meetings', id);
    },
  },

  emails: {
    async getByRoadmapItem(roadmapItemId: string): Promise<Email[]> {
      const db = getFirebaseFirestore();
      const q = query(
        collection(db, 'emails'),
        where('roadmapItemId', '==', roadmapItemId)
        // Temporarily removed orderBy to avoid composite index requirement
        // orderBy('date', 'desc')
      );
      const snapshot = await getDocs(q);
      return snapshot.docs.map((doc) => ({
        id: doc.id,
        ...doc.data(),
        createdAt: timestampToISO(doc.data().createdAt),
      })) as Email[];
    },
    async create(data: Omit<Email, 'id' | 'createdAt'>): Promise<Email> {
      return create<Email>('emails', data);
    },
    async update(id: string, updates: Partial<Omit<Email, 'id' | 'roadmapItemId' | 'createdAt'>>): Promise<Email> {
      return update<Email>('emails', id, updates);
    },
    async delete(id: string): Promise<void> {
      await deleteDoc_('emails', id);
    },
  },

  chats: {
    async getByRoadmapItem(roadmapItemId: string): Promise<Chat[]> {
      const db = getFirebaseFirestore();
      const q = query(
        collection(db, 'chats'),
        where('roadmapItemId', '==', roadmapItemId)
        // Temporarily removed orderBy to avoid composite index requirement
        // orderBy('date', 'desc')
      );
      const snapshot = await getDocs(q);
      return snapshot.docs.map((doc) => ({
        id: doc.id,
        ...doc.data(),
        createdAt: timestampToISO(doc.data().createdAt),
      })) as Chat[];
    },
    async create(data: Omit<Chat, 'id' | 'createdAt'>): Promise<Chat> {
      return create<Chat>('chats', data);
    },
    async update(id: string, updates: Partial<Omit<Chat, 'id' | 'roadmapItemId' | 'createdAt'>>): Promise<Chat> {
      return update<Chat>('chats', id, updates);
    },
    async delete(id: string): Promise<void> {
      await deleteDoc_('chats', id);
    },
  },

  aiRequirements: {
    async getByRoadmapItem(roadmapItemId: string): Promise<AIRequirement[]> {
      const db = getFirebaseFirestore();
      const q = query(
        collection(db, 'aiRequirements'),
        where('roadmapItemId', '==', roadmapItemId)
      );
      const snapshot = await getDocs(q);
      return snapshot.docs.map((doc) => ({
        id: doc.id,
        ...doc.data(),
        extractedAt: timestampToISO(doc.data().extractedAt),
      })) as AIRequirement[];
    },
    async create(data: Omit<AIRequirement, 'id' | 'extractedAt'>): Promise<AIRequirement> {
      return create<AIRequirement>('aiRequirements', { ...data, extractedAt: new Date().toISOString() });
    },
    async delete(id: string): Promise<void> {
      await deleteDoc_('aiRequirements', id);
    },
  },

  gaps: {
    async getByRoadmapItem(roadmapItemId: string): Promise<Gap[]> {
      return getAll<Gap>('gaps', [where('roadmapItemId', '==', roadmapItemId)]);
    },
    async create(data: Omit<Gap, 'id' | 'createdAt' | 'updatedAt'>): Promise<Gap> {
      return create<Gap>('gaps', data);
    },
    async update(id: string, data: Partial<Gap>): Promise<Gap> {
      return update<Gap>('gaps', id, data);
    },
    async delete(id: string): Promise<void> {
      await deleteDoc_('gaps', id);
    },
  },

  confusions: {
    async getByRoadmapItem(roadmapItemId: string): Promise<Confusion[]> {
      return getAll<Confusion>('confusions', [where('roadmapItemId', '==', roadmapItemId)]);
    },
    async create(data: Omit<Confusion, 'id' | 'createdAt' | 'updatedAt'>): Promise<Confusion> {
      return create<Confusion>('confusions', data);
    },
    async update(id: string, data: Partial<Confusion>): Promise<Confusion> {
      return update<Confusion>('confusions', id, data);
    },
    async delete(id: string): Promise<void> {
      await deleteDoc_('confusions', id);
    },
  },

  suggestions: {
    async getByRoadmapItem(roadmapItemId: string): Promise<Suggestion[]> {
      return getAll<Suggestion>('suggestions', [where('roadmapItemId', '==', roadmapItemId)]);
    },
    async create(data: Omit<Suggestion, 'id' | 'createdAt' | 'updatedAt'>): Promise<Suggestion> {
      return create<Suggestion>('suggestions', data);
    },
    async update(id: string, data: Partial<Suggestion>): Promise<Suggestion> {
      return update<Suggestion>('suggestions', id, data);
    },
    async delete(id: string): Promise<void> {
      await deleteDoc_('suggestions', id);
    },
  },

  securityRequirements: {
    async getByRoadmapItem(roadmapItemId: string): Promise<SecurityRequirement[]> {
      return getAll<SecurityRequirement>('securityRequirements', [where('roadmapItemId', '==', roadmapItemId)]);
    },
    async create(data: Omit<SecurityRequirement, 'id' | 'createdAt' | 'updatedAt'>): Promise<SecurityRequirement> {
      return create<SecurityRequirement>('securityRequirements', data);
    },
    async update(id: string, data: Partial<SecurityRequirement>): Promise<SecurityRequirement> {
      return update<SecurityRequirement>('securityRequirements', id, data);
    },
    async delete(id: string): Promise<void> {
      await deleteDoc_('securityRequirements', id);
    },
  },

  threats: {
    async getByRoadmapItem(roadmapItemId: string): Promise<Threat[]> {
      return getAll<Threat>('threats', [where('roadmapItemId', '==', roadmapItemId)]);
    },
    async create(data: Omit<Threat, 'id' | 'createdAt' | 'updatedAt'>): Promise<Threat> {
      return create<Threat>('threats', data);
    },
    async update(id: string, data: Partial<Threat>): Promise<Threat> {
      return update<Threat>('threats', id, data);
    },
    async delete(id: string): Promise<void> {
      await deleteDoc_('threats', id);
    },
  },

  nfrs: {
    async getByRoadmapItem(roadmapItemId: string): Promise<NonFunctionalRequirement[]> {
      return getAll<NonFunctionalRequirement>('nfrs', [where('roadmapItemId', '==', roadmapItemId)]);
    },
    async create(data: Omit<NonFunctionalRequirement, 'id' | 'createdAt' | 'updatedAt'>): Promise<NonFunctionalRequirement> {
      return create<NonFunctionalRequirement>('nfrs', data);
    },
    async update(id: string, data: Partial<NonFunctionalRequirement>): Promise<NonFunctionalRequirement> {
      return update<NonFunctionalRequirement>('nfrs', id, data);
    },
    async delete(id: string): Promise<void> {
      await deleteDoc_('nfrs', id);
    },
  },

  cloudGovernance: {
    async getByRoadmapItem(roadmapItemId: string): Promise<CloudGovernance[]> {
      return getAll<CloudGovernance>('cloudGovernance', [where('roadmapItemId', '==', roadmapItemId)]);
    },
    async create(data: Omit<CloudGovernance, 'id' | 'createdAt' | 'updatedAt'>): Promise<CloudGovernance> {
      return create<CloudGovernance>('cloudGovernance', data);
    },
    async update(id: string, data: Partial<CloudGovernance>): Promise<CloudGovernance> {
      return update<CloudGovernance>('cloudGovernance', id, data);
    },
    async delete(id: string): Promise<void> {
      await deleteDoc_('cloudGovernance', id);
    },
  },

  formalRequirements: {
    async getByRoadmapItem(roadmapItemId: string): Promise<FormalRequirement[]> {
      return getAll<FormalRequirement>('formalRequirements', [where('roadmapItemId', '==', roadmapItemId)]);
    },
    async getByProject(projectId: string): Promise<FormalRequirement[]> {
      return getAll<FormalRequirement>('formalRequirements', [where('projectId', '==', projectId)]);
    },
    async create(data: Omit<FormalRequirement, 'id' | 'createdAt' | 'updatedAt'>): Promise<FormalRequirement> {
      return create<FormalRequirement>('formalRequirements', data);
    },
    async update(id: string, data: Partial<FormalRequirement>): Promise<FormalRequirement> {
      return update<FormalRequirement>('formalRequirements', id, data);
    },
    async delete(id: string): Promise<void> {
      await deleteDoc_('formalRequirements', id);
    },
  },

  pis: {
    async getByProject(projectId: string): Promise<PI[]> {
      return getAll<PI>('pis', [where('projectId', '==', projectId)]);
    },
    async create(data: Omit<PI, 'id' | 'createdAt' | 'updatedAt'>): Promise<PI> {
      return create<PI>('pis', data);
    },
    async update(id: string, data: Partial<PI>): Promise<PI> {
      return update<PI>('pis', id, data);
    },
    async delete(id: string): Promise<void> {
      await deleteDoc_('pis', id);
    },
  },

  sprints: {
    async getByProject(projectId: string): Promise<Sprint[]> {
      return getAll<Sprint>('sprints', [where('projectId', '==', projectId)]);
    },
    async create(data: Omit<Sprint, 'id' | 'createdAt' | 'updatedAt'>): Promise<Sprint> {
      return create<Sprint>('sprints', data);
    },
    async update(id: string, data: Partial<Sprint>): Promise<Sprint> {
      return update<Sprint>('sprints', id, data);
    },
    async delete(id: string): Promise<void> {
      await deleteDoc_('sprints', id);
    },
  },

  requirements: {
    async getByProject(projectId: string): Promise<Requirement[]> {
      return getAll<Requirement>('requirements', [where('projectId', '==', projectId)]);
    },
    async create(data: Omit<Requirement, 'id' | 'createdAt' | 'updatedAt'>): Promise<Requirement> {
      return create<Requirement>('requirements', data);
    },
    async bulkCreate(projectId: string, reqs: Array<Omit<Requirement, 'id' | 'projectId' | 'createdAt' | 'updatedAt'>>): Promise<Requirement[]> {
      return Promise.all(reqs.map((r) => create<Requirement>('requirements', { ...r, projectId })));
    },
    async update(id: string, data: Partial<Requirement>): Promise<Requirement> {
      return update<Requirement>('requirements', id, data);
    },
    async delete(id: string): Promise<void> {
      await deleteDoc_('requirements', id);
    },
  },

  stories: {
    async getByProject(projectId: string): Promise<Story[]> {
      return getAll<Story>('stories', [where('projectId', '==', projectId)]);
    },
    async getByRequirement(requirementId: string): Promise<Story[]> {
      return getAll<Story>('stories', [where('requirementId', '==', requirementId)]);
    },
    async create(data: Omit<Story, 'id' | 'createdAt' | 'updatedAt'>): Promise<Story> {
      return create<Story>('stories', data);
    },
    async bulkCreate(stories: Array<Omit<Story, 'id' | 'createdAt' | 'updatedAt'>>): Promise<Story[]> {
      return Promise.all(stories.map(s => create<Story>('stories', s)));
    },
    async update(id: string, data: Partial<Story>): Promise<Story> {
      return update<Story>('stories', id, data);
    },
    async delete(id: string): Promise<void> {
      await deleteDoc_('stories', id);
    },
  },

  workflowRequests: {
    async getRaisedBy(userId: string): Promise<WorkflowRequest[]> {
      return getAll<WorkflowRequest>('workflowRequests', [where('raiserId', '==', userId)]);
    },
    async getAssignedToRole(role: string): Promise<WorkflowRequest[]> {
      return getAll<WorkflowRequest>('workflowRequests', [where('targetRole', '==', role)]);
    },
    async getAssignedToUser(userId: string): Promise<WorkflowRequest[]> {
      return getAll<WorkflowRequest>('workflowRequests', [where('targetUserId', '==', userId)]);
    },
    async create(data: Omit<WorkflowRequest, 'id' | 'createdAt' | 'updatedAt'>): Promise<WorkflowRequest> {
      return create<WorkflowRequest>('workflowRequests', data);
    },
    async update(id: string, data: Partial<WorkflowRequest>): Promise<WorkflowRequest> {
      return update<WorkflowRequest>('workflowRequests', id, data);
    },
    async delete(id: string): Promise<void> {
      await deleteDoc_('workflowRequests', id);
    },
  },

  tradeoffAnalyses: {
    async getByProject(projectId: string): Promise<TradeoffAnalysis[]> {
      return getAll<TradeoffAnalysis>('tradeoffAnalyses', [where('projectId', '==', projectId)]);
    },
    async getById(id: string): Promise<TradeoffAnalysis> {
      return getById<TradeoffAnalysis>('tradeoffAnalyses', id);
    },
    async create(data: Omit<TradeoffAnalysis, 'id' | 'createdAt' | 'updatedAt'>): Promise<TradeoffAnalysis> {
      return create<TradeoffAnalysis>('tradeoffAnalyses', data);
    },
    async update(id: string, data: Partial<TradeoffAnalysis>): Promise<TradeoffAnalysis> {
      return update<TradeoffAnalysis>('tradeoffAnalyses', id, data);
    },
    async delete(id: string): Promise<void> {
      await deleteDoc_('tradeoffAnalyses', id);
    },
  },

  users: {
    async getAll(): Promise<UserProfile[]> {
      return getAll<UserProfile>(USERS_COLLECTION);
    },
    async getById(id: string): Promise<UserProfile> {
      return getById<UserProfile>(USERS_COLLECTION, id);
    },
    async update(id: string, data: Partial<UserProfile>): Promise<UserProfile> {
      return update<UserProfile>(USERS_COLLECTION, id, data);
    },
  },

  invites: {
    async getAll(): Promise<Invite[]> {
      return getAll<Invite>('invites');
    },
    async getByToken(token: string): Promise<Invite | null> {
      const res = await getAll<Invite>('invites', [where('token', '==', token)]);
      return res[0] || null;
    },
    async create(data: Omit<Invite, 'id' | 'createdAt' | 'updatedAt'>): Promise<Invite> {
      return create<Invite>('invites', data);
    },
    async update(id: string, data: Partial<Invite>): Promise<Invite> {
      return update<Invite>('invites', id, data);
    },
    async delete(id: string): Promise<void> {
      await deleteDoc_('invites', id);
    },
  },

  conflicts: {
    async getByRoadmapItem(roadmapItemId: string): Promise<Conflict[]> {
      return getAll<Conflict>('conflicts', [where('roadmapItemId', '==', roadmapItemId)]);
    },
    async create(data: Omit<Conflict, 'id' | 'createdAt' | 'updatedAt'>): Promise<Conflict> {
      return create<Conflict>('conflicts', data);
    },
    async update(id: string, data: Partial<Conflict>): Promise<Conflict> {
      return update<Conflict>('conflicts', id, data);
    },
    async delete(id: string): Promise<void> {
      await deleteDoc_('conflicts', id);
    },
  },

  sharedContentViews: {
    async getByRoadmapItem(roadmapItemId: string): Promise<SharedContentView[]> {
      return getAll<SharedContentView>('sharedContentViews', [where('roadmapItemId', '==', roadmapItemId)]);
    },
    async getByUser(userId: string): Promise<SharedContentView[]> {
      return getAll<SharedContentView>('sharedContentViews', [where('sharedWith', 'array-contains', userId)]);
    },
    async create(data: Omit<SharedContentView, 'id' | 'createdAt' | 'updatedAt'>): Promise<SharedContentView> {
      return create<SharedContentView>('sharedContentViews', data);
    },
    async update(id: string, data: Partial<SharedContentView>): Promise<SharedContentView> {
      return update<SharedContentView>('sharedContentViews', id, data);
    },
    async delete(id: string): Promise<void> {
      await deleteDoc_('sharedContentViews', id);
    },
  },

  projectMembers: {
    async getByProject(projectId: string): Promise<ProjectMember[]> {
      return getAll<ProjectMember>('projectMembers', [where('projectId', '==', projectId)]);
    },
    async getByUser(userId: string): Promise<ProjectMember[]> {
      return getAll<ProjectMember>('projectMembers', [where('userId', '==', userId)]);
    },
    async create(data: Omit<ProjectMember, 'id' | 'addedAt' | 'updatedAt'>): Promise<ProjectMember> {
      const now = new Date().toISOString();
      const firestore = getFirebaseFirestore();
      const ref = doc(collection(firestore, 'projectMembers'));
      const record: ProjectMember = { ...data, id: ref.id, addedAt: now, updatedAt: now };
      await setDoc(ref, record);
      return record;
    },
    async update(id: string, data: { projectRoles: ProjectRole[] }): Promise<ProjectMember> {
      return update<ProjectMember>('projectMembers', id, data);
    },
    async delete(id: string): Promise<void> {
      await deleteDoc_('projectMembers', id);
    },
  },

  designerDocuments: {
    async getByProject(projectId: string): Promise<DesignerDocument[]> {
      return getAll<DesignerDocument>('designerDocuments', [where('projectId', '==', projectId)]);
    },
    async getById(id: string): Promise<DesignerDocument | null> {
      return getOne<DesignerDocument>('designerDocuments', id);
    },
    async create(data: Omit<DesignerDocument, 'id' | 'createdAt' | 'updatedAt'>): Promise<DesignerDocument> {
      const now = new Date().toISOString();
      const firestore = getFirebaseFirestore();
      const ref = doc(collection(firestore, 'designerDocuments'));
      const record: DesignerDocument = { ...data, id: ref.id, createdAt: now, updatedAt: now };
      await setDoc(ref, record);
      return record;
    },
    async update(id: string, data: Partial<Omit<DesignerDocument, 'id' | 'createdAt'>>): Promise<DesignerDocument> {
      return update<DesignerDocument>('designerDocuments', id, data);
    },
    async delete(id: string): Promise<void> {
      await deleteDoc_('designerDocuments', id);
    },
  },
};
