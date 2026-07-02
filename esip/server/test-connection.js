import { PrismaClient } from '@prisma/client';
import dotenv from 'dotenv';

// Load environment variables
dotenv.config();

const prisma = new PrismaClient({
  log: ['query', 'info', 'warn', 'error'],
});

async function testConnection() {
  console.log('\n🔗 Testing Supabase PostgreSQL Connection...\n');
  
  try {
    // Test connection
    await prisma.$connect();
    console.log('✅ Successfully connected to Supabase PostgreSQL!\n');
    
    // Get database info
    console.log('📊 Database Statistics:');
    console.log('─────────────────────────');
    
    const userCount = await prisma.user.count();
    const teamCount = await prisma.team.count();
    const projectCount = await prisma.project.count();
    const wbsTaskCount = await prisma.wbsTask.count();
    
    console.log(`Users:           ${userCount}`);
    console.log(`Teams:           ${teamCount}`);
    console.log(`Projects:        ${projectCount}`);
    console.log(`WBS Tasks:       ${wbsTaskCount}`);
    console.log('─────────────────────────\n');
    
    // Test query execution
    console.log('🔍 Testing Query Execution...');
    const tables = await prisma.$queryRaw`
      SELECT table_name 
      FROM information_schema.tables 
      WHERE table_schema = 'public'
      ORDER BY table_name;
    `;
    
    console.log('✅ Query executed successfully!\n');
    console.log('📋 Database Tables:');
    tables.forEach((table, index) => {
      console.log(`   ${index + 1}. ${table.table_name}`);
    });
    
    console.log('\n✨ All tests passed! Your Supabase connection is working perfectly.\n');
    
    await prisma.$disconnect();
    process.exit(0);
    
  } catch (error) {
    console.error('\n❌ Connection Failed!\n');
    console.error('Error Details:');
    console.error('─────────────────────────');
    console.error(error.message);
    console.error('\n💡 Troubleshooting:');
    console.error('   1. Check your .env file exists and has DATABASE_URL');
    console.error('   2. Verify your Supabase credentials are correct');
    console.error('   3. Make sure you ran: npm run db:push');
    console.error('   4. Check that your IP is allowed in Supabase settings\n');
    
    await prisma.$disconnect();
    process.exit(1);
  }
}

testConnection();
