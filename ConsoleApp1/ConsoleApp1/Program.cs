using System;
using System.Collections.Generic;
using System.IO;
using System.Linq;
using System.Net.Http.Headers;
using System.Reflection;
using System.Text;
using System.Text.Json;
using System.Text.Json.Nodes;
using System.Text.RegularExpressions;
using YamlDotNet.Serialization;
using YamlDotNet.Serialization.NamingConventions;

namespace RepairLinkPipelineAnalyzer
{
    // API Response Model for Agent Queue (Deployment Group)
    public class AgentQueueResponse
    {
        public int count { get; set; }
        public List<AgentQueue> value { get; set; }
    }

    public class AgentQueue
    {
        public int id { get; set; }
        public string name { get; set; }
        public AgentPool pool { get; set; }
    }

    public class AgentPool
    {
        public int id { get; set; }
        public string name { get; set; }
        public bool isHosted { get; set; }
        public string poolType { get; set; }
    }

    // API Response Model for Deployment Targets (Actual Servers)
    public class DeploymentTargetsResponse
    {
        public int count { get; set; }
        public List<DeploymentTargetInfo> value { get; set; }
    }

    public class DeploymentTargetInfo
    {
        public int id { get; set; }
        public AgentInfo agent { get; set; }
    }


    public class AgentInfo
    {
        public string name { get; set; }
        public string status { get; set; }
    }

    public class AgentStatus
    {
        public string status { get; set; } // e.g., "Online" or "Offline"
        public string version { get; set; }
        // ... other properties if needed
    }

    // ---------- Models ----------
    public class BuildPipeline
    {
        public string FilePath { get; set; }
        public string Name { get; set; }
        public string Folder { get; set; }
        public List<string> ProducedArtifacts { get; set; } = new();
        public List<string> Repositories { get; set; } = new();
        public string TriggerBranch { get; set; }
    }

    public class ReleasePipeline
    {
        public string FilePath { get; set; }
        public string Name { get; set; }
        public List<ReleaseEnvironment> Environments { get; set; } = new();
        public List<ArtifactReference> Artifacts { get; set; } = new();
        public List<DeploymentTarget> DeploymentTargets { get; set; } = new();
    }

    public class ReleaseEnvironment
    {
        public string Name { get; set; }
        public List<DeploymentTask> Tasks { get; set; } = new();
    }

    public class DeploymentTask
    {
        public string TaskType { get; set; }   // e.g., "IISWebAppDeployment", "AzureWebApp"
        public Dictionary<string, string> Properties { get; set; } = new();
    }

    public class ArtifactReference
    {
        public string Alias { get; set; }
        public string BuildDefinitionId { get; set; }
        public string BuildName { get; set; } // resolved later
    }

    public class DeploymentTarget
    {
        public string ReleaseName { get; set; }
        public string Environment { get; set; }
        public string ServerName { get; set; }
        public string PhysicalPath { get; set; }
        public string SiteName { get; set; }
        public string AdditionalInfo { get; set; }
    }

    // ---------- Main Analyzer ----------
    class Program
    {
        //static async Task Main(string[] args)
        //{
        //    Console.WriteLine("🔍 RepairLink Pipeline Analyzer Started\n");

        //    // 🔧 CHANGE THESE PATHS TO YOUR LOCAL FOLDERS
        //    string buildsFolder = @"C:\RepairLink\pipelines\repairlink\build";
        //    string releasesFolder = @"C:\RepairLink\pipelines\repairlink\release";
        //    string outputCsv = @"C:\RepairLink\pipelines\output\RepairLink_DeploymentMap.csv";

        //    // Azure DevOps API Settings - YOU MUST UPDATE THESE
        //    string pat = "4gXIWscStXiCd4VGmKqjAev8OSEvJApxa0tY3ocFioblPs6jJUAhJQQJ99CEACAAAAAbdZKXAAASAZDO32n1"; // The PAT you generated
        //    string organization = "oec-vsts";
        //    string project = "RepairLink";
        //    int deploymentGroupId = 2031; // Tumhare URL se mgid

        //    List<string> servers = await GetServersForDeploymentGroup(organization, project, deploymentGroupId, pat);
        //    Console.WriteLine($"Servers in Deployment Group {deploymentGroupId}: {string.Join(", ", servers)}");
        //    // 1. Load Build Pipelines
        //    var builds = LoadBuildPipelines(buildsFolder);
        //    Console.WriteLine($"📦 Loaded {builds.Count} build pipelines");

        //    // 2. NEW: Fetch mapping of QueueId to actual server names
        //    Console.WriteLine("\n🔌 Connecting to Azure DevOps API to fetch server information...");
        //    var queueIdToGroupName = await GetQueueIdToDeploymentGroupMapping(organization, project, pat);
        //    //var queueIdToServers = await GetQueueIdToServerNamesMapping(organization, project, pat, queueIdToGroupName);
        //    //Console.WriteLine($"✅ Fetched mapping for {queueIdToServers.Count} deployment groups\n");


        //    // 2. Load Release Pipelines & extract deployment targets
        //    var releases = LoadReleasePipelines(releasesFolder, builds);
        //    Console.WriteLine($"🚀 Loaded {releases.Count} release pipelines");

        //    // 3. Generate holistic report
        //    GenerateReport(releases, builds, outputCsv);
        //    Console.WriteLine($"\n✅ Report saved to: {outputCsv}");

        //    // 4. Extra insights
        //    ShowEcosystemInsights(builds, releases);
        //}
        static async Task Main(string[] args)
        {
            Console.WriteLine("🔍 RepairLink Pipeline Analyzer Started\n");

            string buildsFolder = @"C:\RepairLink\pipelines\repairlink\build";   // Update as needed
            string releasesFolder = @"C:\RepairLink\pipelines\repairlink\release";
            string outputCsv = @"C:\RepairLink\pipelines\output\RepairLink_DeploymentMap.csv";

            string pat = "4gXIWscStXiCd4VGmKqjAev8OSEvJApxa0tY3ocFioblPs6jJUAhJQQJ99CEACAAAAAbdZKXAAASAZDO32n1"; // The PAT you generated
            string organization = "oec-vsts";
            string project = "RepairLink";

            var builds = LoadBuildPipelines(buildsFolder);
            Console.WriteLine($"📦 Loaded {builds.Count} build pipelines");

            //var releases = await LoadReleasePipelines(releasesFolder, builds, organization, project, pat);
            //Console.WriteLine($"🚀 Loaded {releases.Count} release pipelines");
            // Fetch releases directly from API (no corrupted files)
            Console.WriteLine("🔌 Fetching release definitions from Azure DevOps API...");
            var releases = await FetchReleasePipelinesFromApi(organization, project, pat, builds);
            Console.WriteLine($"🚀 Fetched {releases.Count} release pipelines from API");


            GenerateReport(releases, builds, outputCsv);
            Console.WriteLine($"\n✅ Report saved to: {outputCsv}");

            ShowEcosystemInsights(builds, releases);
        }

        static async Task<List<ReleasePipeline>> LoadReleasePipelines(
    string folder,
    List<BuildPipeline> builds,
    string organization,
    string project,
    string pat)
        {
            var releases = new List<ReleasePipeline>();
            var jsonFiles = Directory.GetFiles(folder, "*.json", SearchOption.AllDirectories);

            // Cache to avoid repeated API calls for same deployment group
            var dgCache = new Dictionary<int, List<string>>();

            foreach (var file in jsonFiles)
            {
                try
                {
                    var json = File.ReadAllText(file);
                    using var doc = JsonDocument.Parse(json);
                    var root = doc.RootElement;

                    var pipeline = new ReleasePipeline
                    {
                        FilePath = file,
                        Name = root.TryGetProperty("name", out var nameProp) ? nameProp.GetString() : Path.GetFileNameWithoutExtension(file)
                    };

                    Console.WriteLine($"\n📄 Processing release: {pipeline.Name}");

                    // Artifacts parsing (same as before)
                    if (root.TryGetProperty("artifacts", out var artifactsProp) && artifactsProp.ValueKind == JsonValueKind.Array)
                    {
                        foreach (var art in artifactsProp.EnumerateArray())
                        {
                            if (art.TryGetProperty("type", out var typeProp) && typeProp.GetString() == "Build")
                            {
                                var alias = art.TryGetProperty("alias", out var a) ? a.GetString() : "";
                                string defId = "";
                                if (art.TryGetProperty("definitionReference", out var defRef) &&
                                    defRef.TryGetProperty("definition", out var def) &&
                                    def.TryGetProperty("id", out var idProp))
                                {
                                    defId = idProp.ValueKind == JsonValueKind.String ? idProp.GetString() : idProp.GetInt32().ToString();
                                }
                                var matchingBuild = builds.FirstOrDefault(b => b.Name.Contains(defId) || (alias != null && b.Name.Contains(alias, StringComparison.OrdinalIgnoreCase)));
                                pipeline.Artifacts.Add(new ArtifactReference
                                {
                                    BuildDefinitionId = defId,
                                    Alias = alias ?? "",
                                    BuildName = matchingBuild?.Name ?? $"Build_{defId}"
                                });
                            }
                        }
                        Console.WriteLine($"   📦 Artifacts: {pipeline.Artifacts.Count}");
                    }

                    // Variables
                    var releaseVars = new Dictionary<string, string>();
                    if (root.TryGetProperty("variables", out var vars) && vars.ValueKind == JsonValueKind.Object)
                    {
                        foreach (var v in vars.EnumerateObject())
                            if (v.Value.TryGetProperty("value", out var val))
                                releaseVars[v.Name] = val.GetString() ?? "";
                    }

                    // Environments
                    if (root.TryGetProperty("environments", out var envs) && envs.ValueKind == JsonValueKind.Array)
                    {
                        foreach (var envNode in envs.EnumerateArray())
                        {
                            var envName = envNode.TryGetProperty("name", out var n) ? n.GetString() : "unknown";
                            Console.WriteLine($"   🌍 {envName}");

                            // Environment variables
                            var envVars = new Dictionary<string, string>(releaseVars);
                            if (envNode.TryGetProperty("variables", out var ev) && ev.ValueKind == JsonValueKind.Object)
                                foreach (var v in ev.EnumerateObject())
                                    if (v.Value.TryGetProperty("value", out var val))
                                        envVars[v.Name] = val.GetString() ?? "";

                            // Deploy phases
                            if (envNode.TryGetProperty("deployPhases", out var phases) && phases.ValueKind == JsonValueKind.Array)
                            {
                                foreach (var phase in phases.EnumerateArray())
                                {
                                    int? deploymentGroupId = null;
                                    if (phase.TryGetProperty("deploymentInput", out var depInput) && depInput.TryGetProperty("queueId", out var qid))
                                    {
                                        deploymentGroupId = qid.GetInt32();
                                    }

                                    if (phase.TryGetProperty("workflowTasks", out var tasks) && tasks.ValueKind == JsonValueKind.Array)
                                    {
                                        foreach (var task in tasks.EnumerateArray())
                                        {
                                            var taskDisplayName = task.TryGetProperty("displayName", out var dn) ? dn.GetString() : "";
                                            var taskId = task.TryGetProperty("taskId", out var tid) ? tid.GetString() : "";
                                            if (string.IsNullOrEmpty(taskId) && task.TryGetProperty("task", out var tObj) && tObj.TryGetProperty("id", out var id2))
                                                taskId = id2.GetString();

                                            var inputs = task.TryGetProperty("inputs", out var inp) ? inp : default;

                                            // Check for any server-related input
                                            string serverNameFromTask = null;
                                            if (inputs.ValueKind != JsonValueKind.Null)
                                            {
                                                if (inputs.TryGetProperty("MachineNames", out var m)) serverNameFromTask = m.GetString();
                                                else if (inputs.TryGetProperty("ComputerName", out var c)) serverNameFromTask = c.GetString();
                                                else if (inputs.TryGetProperty("TargetMachine", out var tm)) serverNameFromTask = tm.GetString();
                                            }

                                            // If no direct server name but we have a deployment group ID, fetch from API
                                            if (string.IsNullOrEmpty(serverNameFromTask) && deploymentGroupId.HasValue)
                                            {
                                                var servers = await GetServersForDeploymentGroup(organization, project, deploymentGroupId.Value, pat, dgCache);
                                                if (servers.Any())
                                                    serverNameFromTask = string.Join(", ", servers);
                                                else
                                                    serverNameFromTask = $"DeploymentGroup_{deploymentGroupId} (No online agents)";
                                            }

                                            // IIS Deployment task (1b467810...)
                                            if (taskId == "1b467810-6725-4b6d-accd-886174c09bba" || taskDisplayName.Contains("Deploy IIS"))
                                            {
                                                var target = new DeploymentTarget
                                                {
                                                    ReleaseName = pipeline.Name,
                                                    Environment = envName,
                                                    ServerName = serverNameFromTask ?? "Unknown",
                                                    AdditionalInfo = "IIS Deploy"
                                                };

                                                if (inputs.ValueKind != JsonValueKind.Null)
                                                {
                                                    if (inputs.TryGetProperty("WebSiteName", out var site))
                                                        target.SiteName = site.GetString() ?? "";
                                                    else if (envVars.ContainsKey("ParentSiteName"))
                                                        target.SiteName = envVars["ParentSiteName"];

                                                    if (inputs.TryGetProperty("VirtualApplication", out var virt))
                                                        target.PhysicalPath = virt.GetString() ?? "";
                                                    if (string.IsNullOrEmpty(target.PhysicalPath) && envVars.ContainsKey("PhysicalPath"))
                                                        target.PhysicalPath = envVars["PhysicalPath"];
                                                }
                                                pipeline.DeploymentTargets.Add(target);
                                                Console.WriteLine($"         ✅ IIS: Site={target.SiteName}, Path={target.PhysicalPath}, Server={target.ServerName}");
                                            }
                                            // Manage IIS task - capture physical path if not already set
                                            else if (taskId == "1b2aec60-dc49-11e6-9b76-63056e018cac" && inputs.ValueKind != JsonValueKind.Null)
                                            {
                                                if (inputs.TryGetProperty("ParentWebsiteNameForApplication", out var parent) &&
                                                    inputs.TryGetProperty("PhysicalPathForApplication", out var phys))
                                                {
                                                    var existing = pipeline.DeploymentTargets.FirstOrDefault(t => t.Environment == envName && string.IsNullOrEmpty(t.SiteName));
                                                    if (existing != null)
                                                    {
                                                        existing.SiteName = parent.GetString() ?? existing.SiteName;
                                                        existing.PhysicalPath = phys.GetString() ?? existing.PhysicalPath;
                                                    }
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                    releases.Add(pipeline);
                    Console.WriteLine($"✅ Loaded: {pipeline.Name} ({pipeline.DeploymentTargets.Count} targets)");
                }
                catch (Exception ex)
                {
                    Console.WriteLine($"❌ Error {file}: {ex.Message}");
                }
            }
            return releases;
        }

        public static async Task<List<ReleasePipeline>> FetchReleasePipelinesFromApi(
    string organization,
    string project,
    string pat,
    List<BuildPipeline> builds)
        {
            var releases = new List<ReleasePipeline>();
            using var httpClient = new HttpClient();

            string url = $"https://vsrm.dev.azure.com/{organization}/{project}/_apis/release/definitions?api-version=7.0";

            var request = new HttpRequestMessage(HttpMethod.Get, url);
            request.Headers.Authorization = new AuthenticationHeaderValue("Basic",
                Convert.ToBase64String(Encoding.ASCII.GetBytes($":{pat}")));

            var response = await httpClient.SendAsync(request);
            response.EnsureSuccessStatusCode();
            var content = await response.Content.ReadAsStringAsync();

            using var doc = JsonDocument.Parse(content);
            var root = doc.RootElement;

            if (root.TryGetProperty("value", out var definitions) && definitions.ValueKind == JsonValueKind.Array)
            {
                foreach (var def in definitions.EnumerateArray())
                {
                    var pipeline = ParseReleaseDefinition(def, builds);
                    releases.Add(pipeline);
                }
            }

            return releases;
        }

        private static ReleasePipeline ParseReleaseDefinition(JsonElement def, List<BuildPipeline> builds)
        {
            var pipeline = new ReleasePipeline
            {
                Name = def.TryGetProperty("name", out var n) ? n.GetString() : "Unknown",
                // FilePath not applicable for API
            };

            // Artifacts
            if (def.TryGetProperty("artifacts", out var artifacts) && artifacts.ValueKind == JsonValueKind.Array)
            {
                foreach (var art in artifacts.EnumerateArray())
                {
                    if (art.TryGetProperty("type", out var type) && type.GetString() == "Build")
                    {
                        var alias = art.TryGetProperty("alias", out var a) ? a.GetString() : "";
                        var defId = "";
                        if (art.TryGetProperty("definitionReference", out var defRef) &&
                            defRef.TryGetProperty("definition", out var definition) &&
                            definition.TryGetProperty("id", out var idProp))
                        {
                            defId = idProp.GetString();
                        }
                        var matchingBuild = builds.FirstOrDefault(b => b.Name.Contains(defId) || b.Name.Contains(alias));
                        pipeline.Artifacts.Add(new ArtifactReference
                        {
                            BuildDefinitionId = defId,
                            Alias = alias,
                            BuildName = matchingBuild?.Name ?? $"Build_{defId}"
                        });
                    }
                }
            }

            // Variables
            var releaseVars = new Dictionary<string, string>();
            if (def.TryGetProperty("variables", out var vars) && vars.ValueKind == JsonValueKind.Object)
            {
                foreach (var v in vars.EnumerateObject())
                    if (v.Value.TryGetProperty("value", out var val))
                        releaseVars[v.Name] = val.GetString() ?? "";
            }

            // Environments
            if (def.TryGetProperty("environments", out var envs) && envs.ValueKind == JsonValueKind.Array)
            {
                foreach (var envNode in envs.EnumerateArray())
                {
                    var envName = envNode.TryGetProperty("name", out var en) ? en.GetString() : "unknown";
                    var env = new ReleaseEnvironment { Name = envName };

                    var envVars = new Dictionary<string, string>(releaseVars);
                    if (envNode.TryGetProperty("variables", out var ev) && ev.ValueKind == JsonValueKind.Object)
                        foreach (var v in ev.EnumerateObject())
                            if (v.Value.TryGetProperty("value", out var val))
                                envVars[v.Name] = val.GetString() ?? "";

                    if (envNode.TryGetProperty("deployPhases", out var phases) && phases.ValueKind == JsonValueKind.Array)
                    {
                        foreach (var phase in phases.EnumerateArray())
                        {
                            int? deploymentGroupId = null;
                            if (phase.TryGetProperty("deploymentInput", out var depInput) && depInput.TryGetProperty("queueId", out var qid))
                                deploymentGroupId = qid.GetInt32();

                            if (phase.TryGetProperty("workflowTasks", out var tasks) && tasks.ValueKind == JsonValueKind.Array)
                            {
                                foreach (var task in tasks.EnumerateArray())
                                {
                                    // Similar parsing as before - extract server names from inputs or via API later
                                    // We'll keep it simple for now
                                }
                            }
                        }
                    }
                    pipeline.Environments.Add(env);
                }
            }

            return pipeline;
        }

        // ---------- Build Pipeline Parsing (YAML) ----------
        static List<BuildPipeline> LoadBuildPipelines(string folder)
        {
            var builds = new List<BuildPipeline>();
            var yamlFiles = Directory.GetFiles(folder, "*.yaml", SearchOption.AllDirectories)
                                     .Concat(Directory.GetFiles(folder, "*.yml", SearchOption.AllDirectories));

            foreach (var file in yamlFiles)
            {
                try
                {
                    var yaml = File.ReadAllText(file);
                    var deserializer = new DeserializerBuilder()
                        .WithNamingConvention(UnderscoredNamingConvention.Instance)
                        .Build();

                    var doc = deserializer.Deserialize<Dictionary<string, object>>(yaml);
                    var pipeline = new BuildPipeline
                    {
                        FilePath = file,
                        Name = Path.GetFileNameWithoutExtension(file),
                        Folder = Path.GetDirectoryName(file)?.Replace(folder, "").TrimStart('\\') ?? ""
                    };

                    // Trigger branch
                    if (doc.ContainsKey("trigger"))
                    {
                        if (doc["trigger"] is Dictionary<object, object> trigger)
                            pipeline.TriggerBranch = trigger.ContainsKey("branches") ? trigger["branches"].ToString() : "main";
                        else if (doc["trigger"] is string branch)
                            pipeline.TriggerBranch = branch;
                    }

                    // Produced artifacts (common pattern)
                    if (doc.ContainsKey("steps"))
                    {
                        var steps = doc["steps"] as List<object>;
                        if (steps != null)
                        {
                            foreach (var step in steps)
                            {
                                var stepDict = step as Dictionary<object, object>;
                                if (stepDict != null && stepDict.ContainsKey("task") && stepDict["task"].ToString().Contains("PublishBuildArtifacts"))
                                {
                                    pipeline.ProducedArtifacts.Add("drop");
                                }
                                if (stepDict != null && stepDict.ContainsKey("task") && stepDict["task"].ToString().Contains("DotNetCoreCLI"))
                                {
                                    var command = stepDict.ContainsKey("command") ? stepDict["command"].ToString() : "";
                                    if (command == "pack" || command == "push")
                                        pipeline.ProducedArtifacts.Add("nuget");
                                }
                            }
                        }
                    }

                    builds.Add(pipeline);
                }
                catch (Exception ex)
                {
                    Console.WriteLine($"⚠️ Error parsing build YAML {file}: {ex.Message}");
                }
            }
            return builds;
        }

        // ---------- Release Pipeline Parsing (JSON) ----------
        static List<ReleasePipeline> LoadReleasePipelines(string folder, List<BuildPipeline> builds)
        {
            var releases = new List<ReleasePipeline>();
            var jsonFiles = Directory.GetFiles(folder, "*.json", SearchOption.AllDirectories);

            foreach (var file in jsonFiles)
            {
                try
                {
                    var json = File.ReadAllText(file);
                    using var doc = JsonDocument.Parse(json);
                    var root = doc.RootElement;

                    var pipeline = new ReleasePipeline
                    {
                        FilePath = file,
                        Name = root.TryGetProperty("name", out var nameProp) ? nameProp.GetString() : Path.GetFileNameWithoutExtension(file)
                    };

                    Console.WriteLine($"\n📄 Processing release: {pipeline.Name}");

                    // Artifacts...
                    if (root.TryGetProperty("artifacts", out var artifactsProp) && artifactsProp.ValueKind == JsonValueKind.Array)
                    {
                        foreach (var art in artifactsProp.EnumerateArray())
                        {
                            var type = art.TryGetProperty("type", out var typeProp) ? typeProp.GetString() : "";
                            if (type == "Build")
                            {
                                var alias = art.TryGetProperty("alias", out var aliasProp) ? aliasProp.GetString() : "";
                                string defId = "";
                                if (art.TryGetProperty("definitionReference", out var defRefProp) &&
                                    defRefProp.TryGetProperty("definition", out var defProp) &&
                                    defProp.TryGetProperty("id", out var idProp))
                                {
                                    defId = idProp.ValueKind == JsonValueKind.String ? idProp.GetString() : idProp.GetInt32().ToString();
                                }
                                var matchingBuild = builds.FirstOrDefault(b =>
                                    b.Name.Contains(defId) ||
                                    (alias != null && b.Name.Contains(alias, StringComparison.OrdinalIgnoreCase)));

                                pipeline.Artifacts.Add(new ArtifactReference
                                {
                                    BuildDefinitionId = defId,
                                    Alias = alias ?? "",
                                    BuildName = matchingBuild?.Name ?? $"Build_{defId}"
                                });
                            }
                        }
                        Console.WriteLine($"   📦 Artifacts: {pipeline.Artifacts.Count}");
                    }

                    // Variables at release level
                    var releaseVars = new Dictionary<string, string>();
                    if (root.TryGetProperty("variables", out var varsProp) && varsProp.ValueKind == JsonValueKind.Object)
                    {
                        foreach (var v in varsProp.EnumerateObject())
                            if (v.Value.TryGetProperty("value", out var val))
                                releaseVars[v.Name] = val.GetString() ?? "";
                    }

                    // Environments
                    if (root.TryGetProperty("environments", out var envsProp) && envsProp.ValueKind == JsonValueKind.Array)
                    {
                        foreach (var envNode in envsProp.EnumerateArray())
                        {
                            var envName = envNode.TryGetProperty("name", out var n) ? n.GetString() : "unknown";
                            Console.WriteLine($"   🌍 {envName}");

                            // Environment variables
                            var envVars = new Dictionary<string, string>(releaseVars);
                            if (envNode.TryGetProperty("variables", out var ev) && ev.ValueKind == JsonValueKind.Object)
                                foreach (var v in ev.EnumerateObject())
                                    if (v.Value.TryGetProperty("value", out var val))
                                        envVars[v.Name] = val.GetString() ?? "";

                            // Deploy phases
                            if (envNode.TryGetProperty("deployPhases", out var phases) && phases.ValueKind == JsonValueKind.Array)
                            {
                                foreach (var phase in phases.EnumerateArray())
                                {
                                    // Get queueId and deployment group name (if any)
                                    string queueId = null;
                                    if (phase.TryGetProperty("deploymentInput", out var depInput) && depInput.TryGetProperty("queueId", out var qid))
                                        queueId = qid.GetInt32().ToString();

                                    if (phase.TryGetProperty("workflowTasks", out var tasks) && tasks.ValueKind == JsonValueKind.Array)
                                    {
                                        foreach (var task in tasks.EnumerateArray())
                                        {
                                            var taskDisplayName = task.TryGetProperty("displayName", out var dn) ? dn.GetString() : "";
                                            var taskId = task.TryGetProperty("taskId", out var tid) ? tid.GetString() : "";
                                            if (string.IsNullOrEmpty(taskId) && task.TryGetProperty("task", out var tObj) && tObj.TryGetProperty("id", out var id2))
                                                taskId = id2.GetString();

                                            var inputs = task.TryGetProperty("inputs", out var inp) ? inp : default;

                                            // Look for any server-related input
                                            string serverNameFromTask = null;
                                            if (inputs.ValueKind != JsonValueKind.Null)
                                            {
                                                if (inputs.TryGetProperty("MachineNames", out var m)) serverNameFromTask = m.GetString();
                                                else if (inputs.TryGetProperty("ComputerName", out var c)) serverNameFromTask = c.GetString();
                                                else if (inputs.TryGetProperty("TargetMachine", out var tm)) serverNameFromTask = tm.GetString();
                                                else if (inputs.TryGetProperty("DscMachineName", out var dsc)) serverNameFromTask = dsc.GetString();
                                            }

                                            // Determine deployment target
                                            if (taskId == "1b467810-6725-4b6d-accd-886174c09bba" || taskDisplayName.Contains("Deploy IIS"))
                                            {
                                                var target = new DeploymentTarget
                                                {
                                                    ReleaseName = pipeline.Name,
                                                    Environment = envName,
                                                    ServerName = serverNameFromTask ?? (queueId != null ? $"DeploymentGroup_{queueId}" : "Unknown"),
                                                    AdditionalInfo = "IIS Deploy"
                                                };

                                                if (inputs.ValueKind != JsonValueKind.Null)
                                                {
                                                    if (inputs.TryGetProperty("WebSiteName", out var site))
                                                        target.SiteName = site.GetString() ?? "";
                                                    else if (envVars.ContainsKey("ParentSiteName"))
                                                        target.SiteName = envVars["ParentSiteName"];

                                                    if (inputs.TryGetProperty("VirtualApplication", out var virt))
                                                        target.PhysicalPath = virt.GetString() ?? "";
                                                    if (string.IsNullOrEmpty(target.PhysicalPath) && envVars.ContainsKey("PhysicalPath"))
                                                        target.PhysicalPath = envVars["PhysicalPath"];
                                                }
                                                pipeline.DeploymentTargets.Add(target);
                                                Console.WriteLine($"         ✅ IIS: Site={target.SiteName}, Path={target.PhysicalPath}, Server={target.ServerName}");
                                            }
                                            else if (taskId == "1b2aec60-dc49-11e6-9b76-63056e018cac" && inputs.ValueKind != JsonValueKind.Null)
                                            {
                                                // Manage IIS - might create app, capture physical path
                                                if (inputs.TryGetProperty("ParentWebsiteNameForApplication", out var parent) &&
                                                    inputs.TryGetProperty("PhysicalPathForApplication", out var phys))
                                                {
                                                    var existing = pipeline.DeploymentTargets.FirstOrDefault(t => t.Environment == envName && string.IsNullOrEmpty(t.SiteName));
                                                    if (existing != null)
                                                    {
                                                        existing.SiteName = parent.GetString() ?? existing.SiteName;
                                                        existing.PhysicalPath = phys.GetString() ?? existing.PhysicalPath;
                                                    }
                                                    else
                                                    {
                                                        pipeline.DeploymentTargets.Add(new DeploymentTarget
                                                        {
                                                            ReleaseName = pipeline.Name,
                                                            Environment = envName,
                                                            ServerName = serverNameFromTask ?? (queueId != null ? $"DeploymentGroup_{queueId}" : "Unknown"),
                                                            SiteName = parent.GetString() ?? "",
                                                            PhysicalPath = phys.GetString() ?? "",
                                                            AdditionalInfo = "IIS Manage"
                                                        });
                                                    }
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }

                    releases.Add(pipeline);
                    Console.WriteLine($"✅ Loaded: {pipeline.Name} ({pipeline.DeploymentTargets.Count} targets)");
                }
                catch (Exception ex)
                {
                    Console.WriteLine($"❌ Error {file}: {ex.Message}");
                }
            }
            return releases;
        }

        // ---------- Report Generation ----------
        static void GenerateReport(List<ReleasePipeline> releases, List<BuildPipeline> builds, string outputPath)
        {
            using var writer = new StreamWriter(outputPath);
            writer.WriteLine("ReleasePipeline,Environment,ServerName,PhysicalPath,SiteName,ArtifactsFromBuilds,AdditionalInfo");

            foreach (var release in releases)
            {
                var artifactBuildNames = string.Join(";", release.Artifacts.Select(a => a.BuildName));
                foreach (var target in release.DeploymentTargets)
                {
                    writer.WriteLine($"\"{EscapeCsv(target.ReleaseName)}\",\"{EscapeCsv(target.Environment)}\",\"{EscapeCsv(target.ServerName)}\",\"{EscapeCsv(target.PhysicalPath)}\",\"{EscapeCsv(target.SiteName)}\",\"{EscapeCsv(artifactBuildNames)}\",\"{EscapeCsv(target.AdditionalInfo)}\"");
                    writer.WriteLine($"\"{target.ReleaseName}\",\"{target.Environment}\",\"{target.ServerName}\",\"{target.PhysicalPath}\",\"{target.SiteName}\",\"{artifactBuildNames}\",\"{target.AdditionalInfo}\"");
                }

                // If no deployment targets found, still list the release with build info
                if (!release.DeploymentTargets.Any())
                {
                    writer.WriteLine($"\"{release.Name}\",\"No target found\",\"?\",\"?\",\"?\",\"{artifactBuildNames}\",\"No deployment task parsed\"");
                }
            }
        }

        static string EscapeCsv(string input) => input?.Replace("\"", "\"\"") ?? "";
        // ---------- Extra Insights for RepairLink ----------
        static void ShowEcosystemInsights(List<BuildPipeline> builds, List<ReleasePipeline> releases)
        {
            Console.WriteLine("\n📊 === REPAIRLINK ECOSYSTEM INSIGHTS ===");

            // 1. Build pipelines that produce deployable artifacts but no release uses them
            var buildsInReleases = releases.SelectMany(r => r.Artifacts.Select(a => a.BuildName)).Distinct().ToList();
            var orphanBuilds = builds.Where(b => !buildsInReleases.Contains(b.Name) && (b.ProducedArtifacts.Contains("drop") || b.ProducedArtifacts.Contains("nuget"))).ToList();
            Console.WriteLine($"\n⚠️ Orphan builds (produce artifacts but never deployed): {orphanBuilds.Count}");
            foreach (var ob in orphanBuilds.Take(10))
                Console.WriteLine($"   - {ob.Name} (artifacts: {string.Join(",", ob.ProducedArtifacts)})");

            // 2. Products (unique release names) and their deployment servers
            var products = releases.Select(r => r.Name).Distinct().ToList();
            Console.WriteLine($"\n✅ Deployed Products (Releases): {products.Count}");
            foreach (var prod in products)
            {
                var servers = releases.Where(r => r.Name == prod).SelectMany(r => r.DeploymentTargets.Select(t => t.ServerName)).Distinct();
                Console.WriteLine($"   - {prod} → servers: {string.Join(", ", servers)}");
            }

            // 3. Shared components (builds that produce nuget packages and are not deployed)
            var sharedComponents = builds.Where(b => b.ProducedArtifacts.Contains("nuget") && !buildsInReleases.Contains(b.Name)).ToList();
            Console.WriteLine($"\n📦 Shared components (NuGet packages, no direct release): {sharedComponents.Count}");
            foreach (var sc in sharedComponents.Take(10))
                Console.WriteLine($"   - {sc.Name}");

            // 4. RepairLink specific (filter by name)
            var repairLinkBuilds = builds.Where(b => b.Name.Contains("RepairLink", StringComparison.OrdinalIgnoreCase)).ToList();
            var repairLinkReleases = releases.Where(r => r.Name.Contains("RepairLink", StringComparison.OrdinalIgnoreCase)).ToList();
            Console.WriteLine($"\n🔧 RepairLink-specific builds: {repairLinkBuilds.Count}, releases: {repairLinkReleases.Count}");
        }

        static async Task<Dictionary<int, string>> GetQueueIdToDeploymentGroupMapping(string organization, string project, string pat)
        {
            var mapping = new Dictionary<int, string>();
            using var httpClient = new HttpClient();

            // 1. First, get all agent queues (deployment pools) in the project
            string queueApiUrl = $"https://dev.azure.com/{organization}/{project}/_apis/distributedtask/queues?api-version=7.1-preview.1";

            var request = new HttpRequestMessage(HttpMethod.Get, queueApiUrl);
            request.Headers.Authorization = new System.Net.Http.Headers.AuthenticationHeaderValue("Basic",
                Convert.ToBase64String(System.Text.Encoding.ASCII.GetBytes(string.Format("{0}:{1}", "", pat))));

            var response = await httpClient.SendAsync(request);
            response.EnsureSuccessStatusCode();
            var content = await response.Content.ReadAsStringAsync();

            var queues = JsonSerializer.Deserialize<AgentQueueResponse>(content);

            if (queues?.value != null)
            {
                foreach (var queue in queues.value)
                {
                    if (queue.pool?.poolType == "deployment") // Filter for deployment groups only
                    {
                        mapping[queue.id] = queue.name;
                    }
                }
            }
            return mapping;
        }

        //static async Task<Dictionary<int, List<string>>> GetQueueIdToServerNamesMapping(string organization, string project, string pat, Dictionary<int, string> queueIdToGroupName)
        //{
        //    var serverMapping = new Dictionary<int, List<string>>();
        //    using var httpClient = new HttpClient();

        //    foreach (var kvp in queueIdToGroupName)
        //    {
        //        var queueId = kvp.Key;
        //        var groupName = kvp.Value;

        //        // 2. For each deployment group, get its targets (actual servers)
        //        // First, get the deployment group ID from its name (queue name is usually the deployment group name)
        //        string getGroupUrl = $"https://dev.azure.com/{organization}/{project}/_apis/distributedtask/deploymentgroups/2031?api-version=7.1-preview.1";

        //        var request = new HttpRequestMessage(HttpMethod.Get, getGroupUrl);
        //        request.Headers.Authorization = new System.Net.Http.Headers.AuthenticationHeaderValue("Basic",
        //            Convert.ToBase64String(System.Text.Encoding.ASCII.GetBytes(string.Format("{0}:{1}", "", pat))));

        //        var response = await httpClient.SendAsync(request);
        //        response.EnsureSuccessStatusCode();
        //        var content = await response.Content.ReadAsStringAsync();

        //        var groupResult = JsonSerializer.Deserialize<AgentQueueResponse>(content);
        //        var deploymentGroupId = groupResult?.value?.FirstOrDefault()?.id ?? 0;

        //        if (deploymentGroupId == 0) continue;

        //        // 3. Get all targets (servers) for this deployment group
        //        string targetsUrl = $"https://dev.azure.com/{organization}/{project}/_apis/distributedtask/deploymentgroups/{deploymentGroupId}/targets?api-version=7.1-preview.1";

        //        request = new HttpRequestMessage(HttpMethod.Get, targetsUrl);
        //        request.Headers.Authorization = new System.Net.Http.Headers.AuthenticationHeaderValue("Basic",
        //            Convert.ToBase64String(System.Text.Encoding.ASCII.GetBytes(string.Format("{0}:{1}", "", pat))));

        //        response = await httpClient.SendAsync(request);
        //        if (response.IsSuccessStatusCode)
        //        {
        //            content = await response.Content.ReadAsStringAsync();
        //            var targetsResult = JsonSerializer.Deserialize<DeploymentTargetsResponse>(content);

        //            var serverNames = targetsResult?.value?.Select(t => t.name).ToList() ?? new List<string>();
        //            serverMapping[queueId] = serverNames;
        //            Console.WriteLine($"      💻 Deployment Group '{groupName}' has {serverNames.Count} servers: {string.Join(", ", serverNames)}");
        //        }
        //    }
        //    return serverMapping;
        //}

        public static async Task<List<string>> GetServersForDeploymentGroup(
    string organization,
    string project,
    int deploymentGroupId,
    string pat,
    Dictionary<int, List<string>> cache = null)
        {
            // Check cache first
            if (cache != null && cache.ContainsKey(deploymentGroupId))
                return cache[deploymentGroupId];

            var serverNames = new List<string>();
            using var httpClient = new HttpClient();

            string url = $"https://dev.azure.com/{organization}/{project}/_apis/distributedtask/deploymentgroups/{deploymentGroupId}/targets?api-version=7.1-preview.1";

            var request = new HttpRequestMessage(HttpMethod.Get, url);
            request.Headers.Authorization = new AuthenticationHeaderValue("Basic",
                Convert.ToBase64String(Encoding.ASCII.GetBytes($":{pat}")));

            var response = await httpClient.SendAsync(request);

            if (response.IsSuccessStatusCode)
            {
                var content = await response.Content.ReadAsStringAsync();
                var targetsResult = JsonSerializer.Deserialize<DeploymentTargetsResponse>(content);

                if (targetsResult?.value != null)
                {
                    foreach (var target in targetsResult.value)
                    {
                        if (target.agent != null && !string.IsNullOrEmpty(target.agent.name) && target.agent.status == "online")
                        {
                            serverNames.Add(target.agent.name);
                        }
                    }
                }
            }
            else
            {
                Console.WriteLine($"   ⚠️ Failed to fetch servers for DG {deploymentGroupId}: {response.StatusCode}");
            }

            // Store in cache
            if (cache != null)
            {
                cache[deploymentGroupId] = serverNames;
            }
            return serverNames;
        }
        public static async Task<List<string>> GetServersForDeploymentGroup(string organization, string project, int deploymentGroupId, string pat)
        {
            var serverNames = new List<string>();
            using var httpClient = new HttpClient();

            string url = $"https://dev.azure.com/{organization}/{project}/_apis/distributedtask/deploymentgroups/{deploymentGroupId}/targets?api-version=7.1-preview.1";

            var request = new HttpRequestMessage(HttpMethod.Get, url);
            request.Headers.Authorization = new AuthenticationHeaderValue("Basic",
                Convert.ToBase64String(Encoding.ASCII.GetBytes($":{pat}")));

            var response = await httpClient.SendAsync(request);

            if (response.IsSuccessStatusCode)
            {
                var content = await response.Content.ReadAsStringAsync();
                var targetsResult = JsonSerializer.Deserialize<DeploymentTargetsResponse>(content);

                if (targetsResult?.value != null)
                {
                    foreach (var target in targetsResult.value)
                    {
                        if (target.agent != null && !string.IsNullOrEmpty(target.agent.name))
                        {
                            serverNames.Add(target.agent.name);
                            Console.WriteLine($"         💻 Found server: {target.agent.name} (Status: {target.agent.status})");
                        }
                    }
                }
            }
            else
            {
                Console.WriteLine($"         ⚠️ Failed to fetch servers: {response.StatusCode}");
            }

            return serverNames;
        }
    }
}