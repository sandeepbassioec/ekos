import { db, auth } from './supabase';

export async function migrateLocalStorageToSupabase() {
  const migrations: { type: string; success: boolean; error?: string }[] = [];

  try {
    // Check if user is authenticated
    const user = await auth.getUser();
    if (!user) {
      console.error('Not authenticated');
      return { success: false, error: 'Not authenticated' };
    }

    // Get all projects from localStorage
    const projectsStorage = localStorage.getItem('estimation-projects-storage');
    if (projectsStorage) {
      try {
        const parsed = JSON.parse(projectsStorage);
        const localProjects = parsed.state?.projects || [];

        console.log(`Found ${localProjects.length} projects in localStorage`);

        for (const localProject of localProjects) {
          try {
            // Check if project already exists in Supabase
            const existingProjects = await db.getProjects();
            const exists = existingProjects.find((p: any) => p.id === localProject.id);

            if (!exists) {
              // Create project in Supabase
              await db.createProject({
                name: localProject.name,
                description: localProject.description,
                status: localProject.status?.toLowerCase() || 'draft',
                estimationTechnique: localProject.selectedMethods?.[0] || 'PERT',
              });
              migrations.push({ type: 'project', success: true });
              console.log(`✅ Migrated project: ${localProject.name}`);
            } else {
              console.log(`⏭️  Skipped existing project: ${localProject.name}`);
            }
          } catch (error: any) {
            console.error(`❌ Failed to migrate project ${localProject.name}:`, error);
            migrations.push({ type: 'project', success: false, error: error.message });
          }
        }
      } catch (error: any) {
        console.error('Failed to parse projects storage:', error);
      }
    }

    // Migrate comprehensive requirements from localStorage
    const keys = Object.keys(localStorage);
    const reqKeys = keys.filter((k) => k.startsWith('comprehensive-requirements-'));

    console.log(`Found ${reqKeys.length} requirement sets in localStorage`);

    for (const key of reqKeys) {
      try {
        const projectId = key.replace('comprehensive-requirements-', '');
        const reqData = localStorage.getItem(key);
        if (!reqData) continue;

        const requirements = JSON.parse(reqData);

        // Get or verify project exists
        const projects = await db.getProjects();
        const project = projects.find((p: any) => p.id === projectId);

        if (!project) {
          console.log(`⏭️  Skipped requirements for non-existent project: ${projectId}`);
          continue;
        }

        // Convert comprehensive requirements to individual requirement records
        const reqsToCreate: any[] = [];

        // Non-functional requirements
        if (requirements.nonFunctional) {
          reqsToCreate.push({
            title: 'Non-Functional Requirements',
            category: 'non-functional',
            description: 'Performance, Scalability, Availability, Reliability, Usability, Maintainability',
            metadata: requirements.nonFunctional,
            status: 'defined',
            priority: 'high',
          });
        }

        // Security requirements
        if (requirements.security) {
          reqsToCreate.push({
            title: 'Security Requirements',
            category: 'security',
            description: 'Authentication, Authorization, Encryption, Compliance, Audit Logging',
            metadata: requirements.security,
            status: 'defined',
            priority: 'critical',
          });
        }

        // Integration requirements
        if (requirements.integration) {
          reqsToCreate.push({
            title: 'Integration Requirements',
            category: 'integration',
            description: 'External APIs, Third-party Services, Legacy Systems',
            metadata: requirements.integration,
            status: 'defined',
            priority: 'high',
          });
        }

        // Data requirements
        if (requirements.data) {
          reqsToCreate.push({
            title: 'Data Requirements',
            category: 'data',
            description: 'Data Model, Migration, Retention, Backup & Recovery',
            metadata: requirements.data,
            status: 'defined',
            priority: 'high',
          });
        }

        // Infrastructure requirements
        if (requirements.infrastructure) {
          reqsToCreate.push({
            title: 'Infrastructure Requirements',
            category: 'infrastructure',
            description: 'Hosting, Database, CDN, CI/CD, Containerization',
            metadata: requirements.infrastructure,
            status: 'defined',
            priority: 'medium',
          });
        }

        if (reqsToCreate.length > 0) {
          await db.bulkCreateRequirements(projectId, reqsToCreate);
          migrations.push({ type: 'requirements', success: true });
          console.log(`✅ Migrated ${reqsToCreate.length} requirement categories for project ${projectId}`);
        }
      } catch (error: any) {
        console.error(`❌ Failed to migrate requirements from ${key}:`, error);
        migrations.push({ type: 'requirements', success: false, error: error.message });
      }
    }

    const summary = {
      total: migrations.length,
      successful: migrations.filter((m) => m.success).length,
      failed: migrations.filter((m) => !m.success).length,
      details: migrations,
    };

    console.log('\n📊 Migration Summary:', summary);
    return { success: true, summary };
  } catch (error: any) {
    console.error('Migration failed:', error);
    return { success: false, error: error.message };
  }
}

// Run migration
if (typeof window !== 'undefined') {
  (window as any).migrateToSupabase = migrateLocalStorageToSupabase;
  console.log('💡 Migration utility loaded. Run: window.migrateToSupabase()');
}