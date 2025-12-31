workspace "Horizon Platform" "Cloud-based IDE platform with AI-powered development, real-time collaboration, and instant deployments" {

    !const BRAND_COLOR "#0F52BA"
    !const IDE_COLOR "#438DD5"
    !const AI_COLOR "#7C3AED"
    !const COLLAB_COLOR "#10B981"
    !const RUNTIME_COLOR "#EF4444"
    !const EXTERNAL_COLOR "#6B7280"
    !const SECURITY_COLOR "#DC2626"

    !impliedRelationships true
    !docs "docs"
    !adrs "adrs"

    model {
        # ========================================
        # People (Actors)
        # ========================================

        developer = person "Developer" "Primary user coding in the cloud IDE" {
            tags "Person"
            properties {
                "persona" "Software developer"
                "skills" "Various programming languages"
                "goals" "Code, collaborate, deploy"
            }
        }

        viewer = person "Viewer" "Read-only access to shared workspaces" {
            tags "Person"
            properties {
                "access_level" "Read-only"
                "use_cases" "Code review, learning"
            }
        }

        enterpriseAdmin = person "Enterprise Admin" "Manages team workspaces and permissions" {
            tags "Person"
            properties {
                "responsibilities" "User management, billing, policies"
                "access_level" "Organization admin"
            }
        }

        devopsEngineer = person "DevOps Engineer" "Deploys and monitors applications" {
            tags "Person"
            properties {
                "tools" "CI/CD, monitoring, infrastructure"
                "focus" "Production deployments"
            }
        }

        # ========================================
        # External Systems
        # ========================================

        group "Code Repositories" {
            github = softwareSystem "GitHub" "Code repository and version control" {
                tags "External,Partner"
                properties {
                    "integration" "OAuth, API"
                    "features" "Import, sync, deploy hooks"
                }
            }

            gitlab = softwareSystem "GitLab" "Alternative code repository" {
                tags "External,Partner"
                properties {
                    "integration" "OAuth, API"
                }
            }
        }

        group "Package Registries" {
            npmRegistry = softwareSystem "npm Registry" "Node.js package registry" {
                tags "External,Partner"
                properties {
                    "packages" "2M+"
                    "protocol" "HTTPS"
                }
            }

            pypi = softwareSystem "PyPI" "Python Package Index" {
                tags "External,Partner"
                properties {
                    "packages" "400K+"
                    "protocol" "HTTPS"
                }
            }

            dockerHub = softwareSystem "Docker Hub" "Container image registry" {
                tags "External,Partner"
                properties {
                    "images" "Millions"
                    "use" "Base images"
                }
            }
        }

        group "AI Providers" {
            anthropicApi = softwareSystem "Anthropic API" "Claude Code Agent SDK" {
                tags "External,AI,Partner"
                properties {
                    "models" "claude-sonnet-4, claude-opus-4"
                    "sdk" "Claude Code Agent SDK"
                    "latency" "1-10s"
                    "auth" "Claude Code OAuth"
                }
            }
        }

        group "Cloud Infrastructure" {
            kubernetesCluster = softwareSystem "Kubernetes Cluster" "Cloud-agnostic container orchestration" {
                tags "External,Cloud"
                properties {
                    "type" "CNCF Kubernetes"
                    "deployment" "Any cloud or on-premises"
                    "components" "Ingress, cert-manager, Helm"
                }
            }
        }

        group "Identity Services" {
            keycloak = softwareSystem "Keycloak" "Self-hosted identity provider" {
                tags "External,Security"
                properties {
                    "protocols" "OAuth 2.0, OIDC, SAML 2.0"
                    "features" "SSO, MFA, social login, user federation"
                    "deployment" "Self-hosted (Docker/K8s)"
                    "license" "Apache 2.0"
                }
            }
        }

        # ========================================
        # Horizon Platform (Main System)
        # ========================================

        horizon = softwareSystem "Horizon Platform" "Cloud-based IDE with AI and collaboration" {
            tags "Software System,Core"

            properties {
                "concurrent_users" "100K+"
                "active_workspaces" "1M+"
                "languages_supported" "50+"
                "ai_completions_per_day" "10M+"
                "perspective_business" "Developer productivity platform with AI pair programming"
                "perspective_technical" "Kubernetes-based with Nix environments and CRDT collaboration"
                "perspective_operations" "Multi-region deployment with autoscaling"
                "perspective_security" "gVisor sandboxing with defense in depth"
            }

            # ========================================
            # Frontend Tier
            # ========================================

            webIDE = container "Web IDE" "Browser-based integrated development environment" "React, Monaco, TypeScript" {
                tags "Container,Frontend"
                properties {
                    "editor" "Monaco (VS Code core)"
                    "state_management" "Zustand"
                    "real_time" "WebSocket, Crosis"
                }

                monacoEditor = component "Monaco Editor" "Code editor with syntax highlighting and IntelliSense" "Monaco" {
                    tags "Component,IDE"
                    properties {
                        "languages" "50+"
                        "features" "Autocomplete, go-to-definition, hover"
                    }
                }

                fileTree = component "File Browser" "Workspace file navigation" "React" {
                    tags "Component,IDE"
                    properties {
                        "features" "Drag-drop, context menu, search"
                    }
                }

                terminalComponent = component "Terminal" "Integrated terminal emulator" "xterm.js" {
                    tags "Component,IDE"
                    properties {
                        "features" "PTY, scrollback, multiple tabs"
                    }
                }

                previewPanel = component "Preview Panel" "Live application preview" "iframe" {
                    tags "Component,IDE"
                    properties {
                        "features" "Hot reload, port detection"
                    }
                }

                collaborationOverlay = component "Collaboration Overlay" "Multiplayer cursors and presence" "Yjs Awareness" {
                    tags "Component,Collaboration"
                    properties {
                        "features" "Cursors, selections, names"
                    }
                }

                aiAssistantPanel = component "AI Assistant" "Chat and code suggestions interface" "React" {
                    tags "Component,AI"
                    properties {
                        "features" "Chat, inline suggestions, commands"
                    }
                }
            }

            mobileApp = container "Mobile App" "Companion app for code review and monitoring" "React Native" {
                tags "Container,Frontend,Mobile"
                properties {
                    "platforms" "iOS, Android"
                    "features" "View code, notifications, deployments"
                }
            }

            # ========================================
            # API Gateway Tier
            # ========================================

            apiGateway = container "API Gateway" "Central entry point for all API requests" "Kong" {
                tags "Container,Gateway"
                properties {
                    "rate_limiting" "10K req/sec"
                    "authentication" "JWT, OAuth2"
                    "protocols" "HTTP/2, WebSocket"
                }
            }

            websocketGateway = container "WebSocket Gateway" "Real-time bidirectional communication" "Go" {
                tags "Container,Gateway,Realtime"
                properties {
                    "connections_per_node" "100K"
                    "protocol" "Crosis (Protobuf)"
                    "heartbeat" "30s"
                }

                connectionManager = component "Connection Manager" "Manages WebSocket lifecycle" "Go" {
                    tags "Component"
                    properties {
                        "reconnection" "Exponential backoff"
                    }
                }

                channelRouter = component "Channel Router" "Routes messages to services" "Go" {
                    tags "Component"
                    properties {
                        "channels" "shell, files, lsp, debug"
                    }
                }

                protobufCodec = component "Protobuf Codec" "Serializes/deserializes Crosis messages" "Go" {
                    tags "Component"
                    properties {
                        "compression" "gzip optional"
                    }
                }
            }

            # ========================================
            # Core Services Tier
            # ========================================

            workspaceService = container "Workspace Service" "Manages workspace metadata and lifecycle" "Go" {
                tags "Container,Core"
                properties {
                    "operations" "CRUD, fork, template"
                    "database" "PostgreSQL"
                }

                workspaceManager = component "Workspace Manager" "CRUD operations for workspaces" "Go" {
                    tags "Component"
                }

                templateEngine = component "Template Engine" "Starter templates and scaffolding" "Go" {
                    tags "Component"
                    properties {
                        "templates" "100+"
                    }
                }

                forkManager = component "Fork Manager" "Workspace cloning and forking" "Go" {
                    tags "Component"
                }

                permissionsController = component "Permissions Controller" "Access control and sharing" "Go" {
                    tags "Component"
                    properties {
                        "roles" "Owner, Editor, Viewer"
                    }
                }
            }

            containerOrchestrator = container "Container Orchestrator" "Manages development container lifecycle" "Go, Kubernetes" {
                tags "Container,Core,Runtime"
                properties {
                    "runtime" "gVisor"
                    "scheduler" "Custom + K8s"
                    "cold_start" "<2s"
                }

                containerScheduler = component "Container Scheduler" "Allocates containers to nodes" "Go" {
                    tags "Component"
                    properties {
                        "algorithm" "Bin packing with affinity"
                    }
                }

                healthMonitor = component "Health Monitor" "Liveness and readiness checks" "Go" {
                    tags "Component"
                    properties {
                        "interval" "10s"
                    }
                }

                resourceManager = component "Resource Manager" "CPU, memory, and disk quotas" "Go" {
                    tags "Component"
                    properties {
                        "limits" "2 CPU, 2GB RAM default"
                    }
                }

                imageBuilder = component "Image Builder" "Builds custom container images" "Go, Kaniko" {
                    tags "Component"
                }

                snapshotManager = component "Snapshot Manager" "Container state persistence" "Go" {
                    tags "Component"
                }
            }

            fileSystemService = container "File System Service" "Virtual file system with real-time sync" "Rust" {
                tags "Container,Core"
                properties {
                    "storage_backend" "S3/GCS"
                    "sync_latency" "<100ms"
                    "max_storage" "256GB per workspace"
                }

                vfsManager = component "VFS Manager" "Virtual file system operations" "Rust, FUSE" {
                    tags "Component"
                }

                syncEngine = component "Sync Engine" "Bidirectional file synchronization" "Rust" {
                    tags "Component"
                    properties {
                        "protocol" "Delta compression"
                    }
                }

                historyTracker = component "History Tracker" "File version history" "Rust" {
                    tags "Component"
                    properties {
                        "retention" "30 days"
                    }
                }

                cacheManager = component "Cache Manager" "Local SSD caching" "Rust" {
                    tags "Component"
                }
            }

            executionEngine = container "Execution Engine" "Code execution and process management" "Go" {
                tags "Container,Core,Runtime"
                properties {
                    "isolation" "Namespace + cgroups"
                    "signals" "Full POSIX"
                }

                processManager = component "Process Manager" "Spawns and manages processes" "Go" {
                    tags "Component"
                }

                outputStreamer = component "Output Streamer" "Streams stdout/stderr" "Go" {
                    tags "Component"
                }

                signalHandler = component "Signal Handler" "POSIX signal forwarding" "Go" {
                    tags "Component"
                }

                portAllocator = component "Port Allocator" "Dynamic port assignment" "Go" {
                    tags "Component"
                    properties {
                        "range" "3000-9999"
                    }
                }
            }

            lspHub = container "LSP Hub" "Language Server Protocol proxy" "TypeScript" {
                tags "Container,Core,IDE"
                properties {
                    "languages" "50+"
                    "lifecycle" "On-demand spawning"
                }

                lspRouter = component "LSP Router" "Routes to language servers" "TypeScript" {
                    tags "Component"
                }

                languageServerPool = component "Language Server Pool" "Manages server instances" "TypeScript" {
                    tags "Component"
                }

                completionAggregator = component "Completion Aggregator" "Combines LSP + AI completions" "TypeScript" {
                    tags "Component,AI"
                }

                diagnosticCollector = component "Diagnostic Collector" "Aggregates errors and warnings" "TypeScript" {
                    tags "Component"
                }
            }

            # ========================================
            # AI Services Tier (Claude Agent SDK)
            # ========================================

            aiAgentOrchestrator = container "AI Agent Orchestrator" "Multi-agent AI coordination" "Python, Anthropic SDK" {
                tags "Container,AI"
                properties {
                    "pattern" "ReAct loops via Claude SDK"
                    "max_iterations" "10"
                    "sdk" "anthropic SDK + custom agent framework"
                }

                managerAgent = component "Manager Agent" "Task decomposition and routing" "Python, Anthropic SDK" {
                    tags "Component,AI"
                    properties {
                        "role" "Orchestration"
                        "pattern" "ReAct loop"
                    }
                }

                editorAgent = component "Editor Agent" "Code generation and modification" "Python, Anthropic SDK" {
                    tags "Component,AI"
                    properties {
                        "role" "Code writing"
                        "tools" "FileReader, FileWriter, CodeGenerator"
                    }
                }

                debuggerAgent = component "Debugger Agent" "Error diagnosis and fixes" "Python, Anthropic SDK" {
                    tags "Component,AI"
                    properties {
                        "role" "Debugging"
                        "tools" "StackTraceAnalyzer, TestRunner, LogViewer"
                    }
                }

                reviewerAgent = component "Reviewer Agent" "Code review and suggestions" "Python, Anthropic SDK" {
                    tags "Component,AI"
                    properties {
                        "role" "Verification"
                        "tools" "StaticAnalyzer, SecurityScanner"
                    }
                }

                agentMemory = component "Agent Memory" "Conversation and context state" "Redis, Qdrant" {
                    tags "Component,AI"
                    properties {
                        "short_term" "Redis"
                        "long_term" "Qdrant vectors"
                    }
                }
            }

            codeIntelligenceService = container "Code Intelligence Service" "AI-powered code understanding" "Python" {
                tags "Container,AI"
                properties {
                    "embeddings" "sentence-transformers (open-source)"
                    "search" "Qdrant vector similarity"
                }

                codeEmbeddings = component "Code Embeddings" "Generates code embeddings" "Python, sentence-transformers" {
                    tags "Component,AI"
                    properties {
                        "model" "all-MiniLM-L6-v2 or code-specific"
                        "dimensions" "384-768"
                    }
                }

                similaritySearch = component "Similarity Search" "Semantic code search" "Python, qdrant-client" {
                    tags "Component,AI"
                    properties {
                        "backend" "Qdrant (open-source)"
                        "features" "Dense + sparse vectors"
                    }
                }

                contextBuilder = component "Context Builder" "RAG for code context" "Python" {
                    tags "Component,AI"
                    properties {
                        "max_tokens" "8000"
                        "strategy" "Semantic + recency"
                    }
                }

                completionRanker = component "Completion Ranker" "Ranks AI suggestions" "Python" {
                    tags "Component,AI"
                }
            }

            aiGateway = container "AI Gateway" "Claude Code Agent SDK interface" "Python, FastAPI" {
                tags "Container,AI"
                properties {
                    "sdk" "Claude Code Agent SDK"
                    "auth" "Claude Code OAuth"
                    "streaming" "SSE"
                }

                claudeSDKClient = component "Claude SDK Client" "Anthropic SDK integration" "Python, anthropic" {
                    tags "Component,AI"
                    properties {
                        "models" "claude-sonnet-4, claude-opus-4"
                        "features" "Tool use, streaming, ReAct loops"
                    }
                }

                rateLimiter = component "Rate Limiter" "Per-user quotas" "Python" {
                    tags "Component"
                }

                responseStreamer = component "Response Streamer" "SSE streaming" "Python" {
                    tags "Component,AI"
                }

                costTracker = component "Cost Tracker" "Usage metering" "Python" {
                    tags "Component"
                }
            }

            # ========================================
            # Collaboration Services Tier
            # ========================================

            collaborationEngine = container "Collaboration Engine" "Real-time collaborative editing" "Rust" {
                tags "Container,Collaboration"
                properties {
                    "algorithm" "CRDT (Yjs)"
                    "latency" "<100ms"
                }

                crdtEngine = component "CRDT Engine" "Conflict-free replicated data" "Rust, Yjs" {
                    tags "Component,Collaboration"
                }

                cursorSync = component "Cursor Sync" "Multiplayer cursor positions" "Rust" {
                    tags "Component,Collaboration"
                }

                presenceService = component "Presence Service" "Online status and awareness" "Rust" {
                    tags "Component,Collaboration"
                }

                operationTransformer = component "Operation Transformer" "Operation broadcast" "Rust" {
                    tags "Component,Collaboration"
                }
            }

            messagingService = container "Messaging Service" "In-IDE communication" "Go" {
                tags "Container,Collaboration"
                properties {
                    "features" "Chat, threads, notifications"
                }
            }

            # ========================================
            # Environment Management Tier
            # ========================================

            nixEnvironmentService = container "Nix Environment Service" "Reproducible development environments" "Go, Nix" {
                tags "Container,Runtime"
                properties {
                    "packages" "80K+"
                    "shared_store" "1TB"
                }

                packageResolver = component "Package Resolver" "Dependency resolution" "Go, Nix" {
                    tags "Component"
                }

                environmentBuilder = component "Environment Builder" "Nix shell builds" "Go, Nix" {
                    tags "Component"
                }

                nixCacheManager = component "Cache Manager" "Shared Nix store" "Go" {
                    tags "Component"
                    properties {
                        "size" "1TB"
                    }
                }

                derivationEvaluator = component "Derivation Evaluator" "Nix expression evaluation" "Go, Nix" {
                    tags "Component"
                }
            }

            secretsManager = container "Secrets Manager" "Secure secrets storage and injection" "Go, Vault" {
                tags "Container,Security"
                properties {
                    "backend" "HashiCorp Vault"
                    "encryption" "AES-256"
                }
            }

            # ========================================
            # Deployment Services Tier
            # ========================================

            deploymentOrchestrator = container "Deployment Orchestrator" "Application deployment management" "Go" {
                tags "Container,Deployment"
                properties {
                    "types" "Autoscale, Static, Reserved VM"
                    "strategy" "Blue-green, Canary"
                }

                deploymentScheduler = component "Deployment Scheduler" "Schedules deployments" "Go" {
                    tags "Component"
                }

                autoscaler = component "Autoscaler" "Horizontal pod autoscaling" "Go" {
                    tags "Component"
                }

                healthChecker = component "Health Checker" "Deployment health" "Go" {
                    tags "Component"
                }

                rollbackManager = component "Rollback Manager" "Rollback on failure" "Go" {
                    tags "Component"
                }

                trafficRouter = component "Traffic Router" "Traffic splitting" "Go" {
                    tags "Component"
                }
            }

            domainManager = container "Domain Manager" "Custom domain and SSL management" "Go" {
                tags "Container,Deployment"
                properties {
                    "ssl" "Let's Encrypt"
                    "dns" "Automatic"
                }

                dnsManager = component "DNS Manager" "DNS record management" "Go" {
                    tags "Component"
                }

                sslProvisioner = component "SSL Provisioner" "Certificate provisioning" "Go, cert-manager" {
                    tags "Component"
                }

                subdomainAllocator = component "Subdomain Allocator" "Preview URL allocation" "Go" {
                    tags "Component"
                }
            }

            # ========================================
            # Data Services Tier
            # ========================================

            horizonDatabase = container "Workspace Database" "Integrated key-value database" "PostgreSQL" {
                tags "Container,Database"
                properties {
                    "type" "Key-value + SQL"
                    "per_workspace" "true"
                }
            }

            objectStorage = container "Object Storage" "File and asset storage" "MinIO (S3-compatible)" {
                tags "Container,Database"
                properties {
                    "capacity" "256GB per workspace"
                    "cdn" "Optional CloudFlare or in-cluster caching"
                    "deployment" "Self-hosted MinIO"
                }
            }

            userDatabase = container "User Database" "User accounts and metadata" "PostgreSQL" {
                tags "Container,Database"
                properties {
                    "tables" "users, workspaces, permissions"
                }
            }

            analyticsDatabase = container "Analytics Database" "Usage analytics and metrics" "ClickHouse" {
                tags "Container,Database"
                properties {
                    "retention" "1 year"
                    "queries" "OLAP"
                }
            }

            vectorDatabase = container "Vector Database" "AI embeddings storage" "Qdrant" {
                tags "Container,Database,AI"
                properties {
                    "type" "Open-source vector database"
                    "vectors" "Billions"
                    "dimensions" "384-1536"
                    "features" "Dense + sparse vectors, payload filtering"
                    "deployment" "Self-hosted or Qdrant Cloud"
                    "license" "Apache 2.0"
                }
            }

            # ========================================
            # Infrastructure Services Tier
            # ========================================

            natsCluster = container "NATS Cluster" "Unified messaging, streaming, and real-time state" "NATS JetStream" {
                tags "Container,Infrastructure,Messaging"
                properties {
                    "cluster_size" "5 nodes"
                    "jetstream_streams" "5 main streams"
                    "kv_buckets" "4 buckets (presence, cursors, sessions, collab_state)"
                    "throughput" "1M msg/sec"
                    "retention" "7-90 days (stream dependent)"
                    "features" "JetStream, KV Store, Object Store, Request-Reply"
                    "protocol" "NATS (TCP 4222), WebSocket (9222)"
                }

                jetStreamEngine = component "JetStream Engine" "Durable stream processing with replay" "NATS JetStream" {
                    tags "Component,Messaging"
                    properties {
                        "storage" "File-based"
                        "replication" "R3"
                        "features" "At-least-once, exactly-once, work queues"
                    }
                }

                kvStore = component "Key-Value Store" "Distributed key-value for ephemeral state" "NATS KV" {
                    tags "Component,State"
                    properties {
                        "consistency" "Strongly consistent"
                        "ttl" "Configurable per bucket"
                        "watch" "Real-time change notifications"
                    }
                }

                objectStore = component "Object Store" "Large object storage for CRDT snapshots" "NATS Object Store" {
                    tags "Component,Storage"
                    properties {
                        "max_size" "10MB per object"
                        "use_case" "CRDT snapshots, document state"
                    }
                }

                pubSubCore = component "Pub/Sub Core" "Core publish-subscribe messaging" "NATS Core" {
                    tags "Component,Messaging"
                    properties {
                        "pattern" "Fire-and-forget, request-reply"
                        "latency" "<1ms"
                    }
                }
            }

            cacheLayer = container "Cache Layer" "Application caching for sessions and data" "Redis Cluster" {
                tags "Container,Infrastructure"
                properties {
                    "memory" "512GB"
                    "operations" "1M ops/sec"
                    "use_cases" "Sessions, workspace metadata, file cache, AI responses"
                }
            }

            searchService = container "Search Service" "Code search and discovery" "Elasticsearch" {
                tags "Container,Infrastructure"
                properties {
                    "indexes" "Code, workspaces, users"
                }
            }

            monitoringStack = container "Monitoring Stack" "Observability platform" "Prometheus, Grafana, Jaeger" {
                tags "Container,Infrastructure"
                properties {
                    "metrics" "Prometheus"
                    "logs" "Loki"
                    "traces" "Jaeger"
                }
            }
        }

        # ========================================
        # Relationships - User to System
        # ========================================

        developer -> webIDE "Writes code in" {
            tags "Sync"
        }
        developer -> mobileApp "Reviews code on" {
            tags "Sync"
        }
        viewer -> webIDE "Views shared workspaces in" {
            tags "Sync"
        }
        enterpriseAdmin -> webIDE "Manages organization via" {
            tags "Sync"
        }
        devopsEngineer -> webIDE "Deploys applications from" {
            tags "Sync"
        }

        # ========================================
        # Relationships - External Systems
        # ========================================

        horizon -> github "Imports repositories from" {
            tags "External"
        }
        horizon -> gitlab "Imports repositories from" {
            tags "External"
        }
        horizon -> npmRegistry "Fetches packages from" {
            tags "External"
        }
        horizon -> pypi "Fetches packages from" {
            tags "External"
        }
        horizon -> dockerHub "Pulls base images from" {
            tags "External"
        }
        horizon -> anthropicApi "AI via Claude Code Agent SDK" {
            tags "External,AI"
        }
        horizon -> kubernetesCluster "Deploys on" {
            tags "External,Cloud"
        }
        horizon -> keycloak "Authenticates users via" {
            tags "External,Security"
        }

        # ========================================
        # Relationships - Frontend to Gateway
        # ========================================

        webIDE -> apiGateway "Makes API requests to" {
            tags "Sync,Internal"
        }
        webIDE -> websocketGateway "Opens WebSocket to" {
            tags "Realtime,Internal"
        }
        mobileApp -> apiGateway "Makes API requests to" {
            tags "Sync,Internal"
        }

        # ========================================
        # Relationships - Gateway to Services
        # ========================================

        apiGateway -> workspaceService "Routes workspace requests to" {
            tags "Sync,Internal"
        }
        apiGateway -> aiGateway "Routes AI requests to" {
            tags "Sync,Internal,AI"
        }
        apiGateway -> deploymentOrchestrator "Routes deployment requests to" {
            tags "Sync,Internal"
        }
        apiGateway -> keycloak "Validates tokens with" {
            tags "Sync,External,Security"
        }

        websocketGateway -> containerOrchestrator "Manages containers via" {
            tags "Realtime,Internal"
        }
        websocketGateway -> collaborationEngine "Syncs edits via" {
            tags "Realtime,Internal,Collaboration"
        }
        websocketGateway -> executionEngine "Streams I/O via" {
            tags "Realtime,Internal"
        }
        websocketGateway -> lspHub "Proxies LSP via" {
            tags "Realtime,Internal"
        }
        websocketGateway -> natsCluster "Bridges WebSocket to NATS" {
            tags "Realtime,Internal"
        }

        # ========================================
        # Relationships - Core Services
        # ========================================

        workspaceService -> userDatabase "Stores metadata in" {
            tags "Sync,Internal"
        }
        workspaceService -> objectStorage "Stores files in" {
            tags "Sync,Internal"
        }
        workspaceService -> cacheLayer "Caches data in" {
            tags "Sync,Internal"
        }

        containerOrchestrator -> nixEnvironmentService "Provisions environments via" {
            tags "Sync,Internal"
        }
        containerOrchestrator -> secretsManager "Injects secrets from" {
            tags "Sync,Internal,Security"
        }
        containerOrchestrator -> natsCluster "Publishes container events to" {
            tags "Async,Internal"
        }

        executionEngine -> containerOrchestrator "Runs in containers from" {
            tags "Sync,Internal"
        }
        executionEngine -> fileSystemService "Accesses files via" {
            tags "Sync,Internal"
        }

        fileSystemService -> objectStorage "Persists files to" {
            tags "Sync,Internal"
        }
        fileSystemService -> cacheLayer "Caches files in" {
            tags "Sync,Internal"
        }

        lspHub -> containerOrchestrator "Spawns language servers in" {
            tags "Sync,Internal"
        }
        lspHub -> codeIntelligenceService "Gets AI completions from" {
            tags "Sync,Internal,AI"
        }

        # ========================================
        # Relationships - AI Services
        # ========================================

        aiAgentOrchestrator -> aiGateway "Requests LLM via" {
            tags "Sync,Internal,AI"
        }
        aiAgentOrchestrator -> codeIntelligenceService "Gets context from" {
            tags "Sync,Internal,AI"
        }
        aiAgentOrchestrator -> fileSystemService "Reads/writes code via" {
            tags "Sync,Internal"
        }
        aiAgentOrchestrator -> executionEngine "Runs tests via" {
            tags "Sync,Internal"
        }

        codeIntelligenceService -> vectorDatabase "Stores embeddings in" {
            tags "Sync,Internal,AI"
        }
        codeIntelligenceService -> searchService "Searches code via" {
            tags "Sync,Internal"
        }

        aiGateway -> anthropicApi "Calls Claude via SDK" {
            tags "Sync,External,AI"
        }
        aiGateway -> cacheLayer "Caches responses in" {
            tags "Sync,Internal"
        }

        # ========================================
        # Relationships - Collaboration Services
        # ========================================

        collaborationEngine -> natsCluster "Broadcasts CRDT operations via" {
            tags "Async,Internal,Collaboration"
        }
        collaborationEngine -> natsCluster "Stores presence in" {
            tags "Sync,Internal"
        }
        collaborationEngine -> objectStorage "Persists document state to" {
            tags "Sync,Internal"
        }

        messagingService -> userDatabase "Stores messages in" {
            tags "Sync,Internal"
        }
        messagingService -> natsCluster "Publishes notifications to" {
            tags "Async,Internal"
        }

        # ========================================
        # Relationships - Deployment Services
        # ========================================

        deploymentOrchestrator -> containerOrchestrator "Deploys containers via" {
            tags "Sync,Internal"
        }
        deploymentOrchestrator -> domainManager "Configures domains via" {
            tags "Sync,Internal"
        }
        deploymentOrchestrator -> natsCluster "Publishes deploy events to" {
            tags "Async,Internal"
        }
        deploymentOrchestrator -> monitoringStack "Reports metrics to" {
            tags "Async,Internal"
        }

        # ========================================
        # Relationships - Environment Services
        # ========================================

        nixEnvironmentService -> objectStorage "Caches derivations in" {
            tags "Sync,Internal"
        }
        nixEnvironmentService -> npmRegistry "Fetches Node packages from" {
            tags "Sync,External"
        }
        nixEnvironmentService -> pypi "Fetches Python packages from" {
            tags "Sync,External"
        }

        # ========================================
        # Relationships - Infrastructure
        # ========================================

        monitoringStack -> containerOrchestrator "Collects metrics from" {
            tags "Async,Internal"
        }
        monitoringStack -> aiGateway "Collects metrics from" {
            tags "Async,Internal"
        }
        monitoringStack -> deploymentOrchestrator "Collects metrics from" {
            tags "Async,Internal"
        }

        # ========================================
        # Deployment Model (Cloud-Agnostic Kubernetes)
        # ========================================

        deploymentEnvironment "Production" {
            k8sCloud = deploymentNode "Kubernetes Cluster" "" "Cloud Provider Agnostic" {
                tags "Cloud"

                primaryRegion = deploymentNode "Primary Region" "" "Region" {
                    tags "Region"

                    k8sCluster = deploymentNode "Kubernetes Namespace: horizon" "" "CNCF Kubernetes" {
                        tags "Kubernetes"

                        apiPool = deploymentNode "API Node Pool" "" "General Purpose Nodes" {
                            tags "NodePool"
                            apiGatewayPod = containerInstance apiGateway
                            websocketGatewayPod = containerInstance websocketGateway
                            workspaceServicePod = containerInstance workspaceService
                        }

                        corePool = deploymentNode "Core Node Pool" "" "High CPU Nodes" {
                            tags "NodePool"
                            containerOrchestratorPod = containerInstance containerOrchestrator
                            fileSystemServicePod = containerInstance fileSystemService
                            executionEnginePod = containerInstance executionEngine
                            lspHubPod = containerInstance lspHub
                        }

                        aiPool = deploymentNode "AI Node Pool" "" "High Memory Nodes" {
                            tags "NodePool,AI"
                            aiAgentOrchestratorPod = containerInstance aiAgentOrchestrator
                            codeIntelligenceServicePod = containerInstance codeIntelligenceService
                            aiGatewayPod = containerInstance aiGateway
                        }

                        collabPool = deploymentNode "Collaboration Node Pool" "" "General Purpose Nodes" {
                            tags "NodePool,Collaboration"
                            collaborationEnginePod = containerInstance collaborationEngine
                            messagingServicePod = containerInstance messagingService
                        }

                        runtimePool = deploymentNode "Runtime Node Pool" "" "gVisor-enabled Nodes" {
                            tags "NodePool,Runtime"
                            nixEnvironmentServicePod = containerInstance nixEnvironmentService
                        }

                        deploymentPool = deploymentNode "Deployment Node Pool" "" "General Purpose Nodes" {
                            tags "NodePool"
                            deploymentOrchestratorPod = containerInstance deploymentOrchestrator
                            domainManagerPod = containerInstance domainManager
                        }
                    }

                    postgresCluster = deploymentNode "PostgreSQL" "" "Bitnami Helm Chart" {
                        tags "Database"
                        userDatabaseInstance = containerInstance userDatabase
                        horizonDatabaseInstance = containerInstance horizonDatabase
                    }

                    redisCluster = deploymentNode "Redis" "" "Redis Helm Chart + Sentinel" {
                        tags "Cache"
                        cacheLayerInstance = containerInstance cacheLayer
                    }

                    minioCluster = deploymentNode "MinIO" "" "S3-Compatible Storage" {
                        tags "Storage"
                        objectStorageInstance = containerInstance objectStorage
                    }

                    qdrantCluster = deploymentNode "Qdrant" "" "Vector Database" {
                        tags "Database,AI"
                        vectorDatabaseInstance = containerInstance vectorDatabase
                    }

                    natsDeployment = deploymentNode "NATS" "" "NATS JetStream Cluster" {
                        tags "Messaging"
                        natsClusterInstance = containerInstance natsCluster
                    }

                    keycloakInstance = deploymentNode "Keycloak" "" "Identity Provider" {
                        tags "Security"
                    }
                }

                secondaryRegion = deploymentNode "Secondary Region" "" "Disaster Recovery" {
                    tags "Region,DR"
                }

                tertiaryRegion = deploymentNode "Tertiary Region" "" "Geographic Distribution" {
                    tags "Region"
                }

                globalServices = deploymentNode "Global Services" "" "Edge/Ingress" {
                    tags "Global"

                    ingressController = deploymentNode "Nginx Ingress" "" "Ingress Controller" {
                        tags "Ingress"
                    }

                    certManager = deploymentNode "cert-manager" "" "TLS via Let's Encrypt" {
                        tags "TLS"
                    }
                }
            }
        }
    }

    views {
        # ========================================
        # System Landscape
        # ========================================

        systemLandscape "horizon-landscape" "Complete Horizon Platform ecosystem" {
            include *
            autoLayout lr 300 100
        }

        # ========================================
        # System Context
        # ========================================

        systemContext horizon "horizon-context" "Horizon Platform in context" {
            include *
            autoLayout lr 250 100
        }

        # ========================================
        # Container View
        # ========================================

        container horizon "horizon-containers" "All platform containers" {
            include *
            autoLayout tb 200 100
        }

        # ========================================
        # Component Views
        # ========================================

        component webIDE "ide-components" "Web IDE components" {
            include *
            autoLayout lr 200 100
        }

        component websocketGateway "gateway-components" "WebSocket Gateway components" {
            include *
            autoLayout lr 200 100
        }

        component containerOrchestrator "orchestrator-components" "Container Orchestrator components" {
            include *
            autoLayout lr 200 100
        }

        component aiAgentOrchestrator "ai-agent-components" "AI Agent Orchestrator components" {
            include *
            autoLayout tb 150 100
        }

        component collaborationEngine "collaboration-components" "Collaboration Engine components" {
            include *
            autoLayout lr 200 100
        }

        component nixEnvironmentService "nix-components" "Nix Environment Service components" {
            include *
            autoLayout lr 200 100
        }

        component deploymentOrchestrator "deployment-components" "Deployment Orchestrator components" {
            include *
            autoLayout lr 200 100
        }

        # ========================================
        # Dynamic Views
        # ========================================

        dynamic horizon "code-execution-flow" "Running code in a workspace" {
            developer -> webIDE "Types code and clicks Run"
            webIDE -> websocketGateway "Sends run command via Crosis"
            websocketGateway -> containerOrchestrator "Starts/connects to container"
            containerOrchestrator -> nixEnvironmentService "Ensures environment ready"
            containerOrchestrator -> executionEngine "Spawns process"
            executionEngine -> fileSystemService "Reads source files"
            executionEngine -> websocketGateway "Streams stdout/stderr"
            websocketGateway -> webIDE "Delivers output"
            webIDE -> developer "Displays results"

            autoLayout lr 250 100
        }

        dynamic horizon "ai-completion-flow" "AI code completion" {
            developer -> webIDE "Types code"
            webIDE -> apiGateway "Requests completion"
            apiGateway -> codeIntelligenceService "Gets context"

            {
                codeIntelligenceService -> vectorDatabase "Finds similar code"
                codeIntelligenceService -> searchService "Searches codebase"
            }

            codeIntelligenceService -> aiGateway "Requests completion with context"
            aiGateway -> anthropicApi "Calls Claude via SDK"
            anthropicApi -> aiGateway "Streams response"
            aiGateway -> codeIntelligenceService "Streams response"
            codeIntelligenceService -> apiGateway "Returns ranked suggestions"
            apiGateway -> webIDE "Delivers completions"
            webIDE -> developer "Shows inline suggestion"

            autoLayout lr 250 100
        }

        dynamic horizon "collaboration-flow" "Real-time collaborative editing" {
            developer -> webIDE "Makes edit"
            webIDE -> websocketGateway "Sends CRDT operation"
            websocketGateway -> collaborationEngine "Processes operation"
            collaborationEngine -> natsCluster "Broadcasts to room via JetStream"

            {
                collaborationEngine -> natsCluster "Updates presence in KV Store"
                collaborationEngine -> objectStorage "Persists state"
            }

            natsCluster -> websocketGateway "Delivers to other clients"
            websocketGateway -> webIDE "Syncs to collaborator"
            webIDE -> viewer "Shows cursor and edit"

            autoLayout lr 250 100
        }

        dynamic horizon "workspace-creation-flow" "Creating a new workspace" {
            developer -> webIDE "Clicks Create Workspace"
            webIDE -> apiGateway "POST /workspaces"
            apiGateway -> workspaceService "Creates workspace"

            {
                workspaceService -> userDatabase "Stores metadata"
                workspaceService -> objectStorage "Initializes storage"
            }

            workspaceService -> containerOrchestrator "Requests container"
            containerOrchestrator -> nixEnvironmentService "Builds environment"

            {
                nixEnvironmentService -> npmRegistry "Fetches dependencies"
                nixEnvironmentService -> pypi "Fetches dependencies"
            }

            containerOrchestrator -> workspaceService "Container ready"
            workspaceService -> apiGateway "Returns workspace"
            apiGateway -> webIDE "Opens IDE"
            webIDE -> developer "Ready to code"

            autoLayout tb 200 100
        }

        dynamic aiAgentOrchestrator "ai-agent-task-flow" "AI Agent executing a task" {
            developer -> webIDE "Describes task in chat"
            webIDE -> aiAgentOrchestrator "Submits task"
            aiAgentOrchestrator -> managerAgent "Decomposes task"
            managerAgent -> editorAgent "Assigns code generation"

            editorAgent -> codeIntelligenceService "Gets context"
            editorAgent -> aiGateway "Generates code"
            editorAgent -> fileSystemService "Writes changes"
            editorAgent -> executionEngine "Runs tests"

            executionEngine -> editorAgent "Returns results"
            editorAgent -> reviewerAgent "Requests review"
            reviewerAgent -> editorAgent "Feedback"

            editorAgent -> managerAgent "Reports completion"
            managerAgent -> aiAgentOrchestrator "Task complete"
            aiAgentOrchestrator -> webIDE "Shows results"
            webIDE -> developer "Displays changes"

            autoLayout tb 200 100
        }

        dynamic horizon "deployment-flow" "Deploying an application" {
            developer -> webIDE "Clicks Deploy"
            webIDE -> apiGateway "POST /deployments"
            apiGateway -> deploymentOrchestrator "Creates deployment"

            deploymentOrchestrator -> containerOrchestrator "Builds image"
            containerOrchestrator -> objectStorage "Pushes image"

            deploymentOrchestrator -> domainManager "Provisions URL"

            {
                domainManager -> kubernetesCluster "Creates Ingress"
                domainManager -> kubernetesCluster "Provisions SSL via cert-manager"
            }

            deploymentOrchestrator -> containerOrchestrator "Deploys pods"
            deploymentOrchestrator -> monitoringStack "Sets up monitoring"

            deploymentOrchestrator -> apiGateway "Returns deployment URL"
            apiGateway -> webIDE "Shows deployment status"
            webIDE -> developer "App is live!"

            autoLayout lr 250 100
        }

        # ========================================
        # Deployment View
        # ========================================

        deployment horizon "Production" "horizon-deployment" "Production deployment on Kubernetes" {
            include *
            autoLayout lr 300 100
        }

        # ========================================
        # Styles
        # ========================================

        themes "https://static.structurizr.com/themes/default/theme.json" "../../freshmart-theme.json"

        styles {
            element "AI" {
                shape Robot
                background "#9370DB"
                color "#FFFFFF"
            }

            element "Collaboration" {
                background "#2ECC71"
                color "#FFFFFF"
            }

            element "Runtime" {
                background "#E74C3C"
                color "#FFFFFF"
            }

            element "Database" {
                shape Cylinder
                background "#008080"
                color "#FFFFFF"
            }

            element "External" {
                background "#999999"
                color "#FFFFFF"
            }

            element "Security" {
                background "#DC143C"
                color "#FFFFFF"
            }

            element "IDE" {
                background "#438DD5"
                color "#FFFFFF"
            }

            element "Gateway" {
                shape Hexagon
                background "#F39C12"
                color "#FFFFFF"
            }

            element "Infrastructure" {
                background "#34495E"
                color "#FFFFFF"
            }

            element "Cloud" {
                shape Cloud
            }

            element "Region" {
                shape Folder
            }

            element "Kubernetes" {
                shape Component
            }

            relationship "Realtime" {
                color "#2ECC71"
                thickness 3
                style Solid
            }

            relationship "Sync" {
                thickness 2
            }

            relationship "Async" {
                style Dashed
                thickness 2
            }

            relationship "External" {
                color "#999999"
                style Dotted
            }

            relationship "AI" {
                color "#9370DB"
                thickness 2
            }

            relationship "Collaboration" {
                color "#2ECC71"
                thickness 2
            }
        }
    }
}
