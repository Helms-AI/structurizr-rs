//! HTML and JavaScript rendering for the explore diagram.

use crate::layout::{ContentType, LayoutConfig, NavItem, generate_page_layout};
use structurizr_core::Workspace;
use super::data::extract_graph_data;

/// Generates the complete HTML page for the explore diagram.
///
/// This creates an interactive force-directed graph visualization of the workspace
/// with pan, zoom, and drag capabilities.
pub fn generate_explore_page_html(
    workspace: &Workspace,
    base_path: &str,
    workspace_id: Option<&str>,
) -> String {
    // Extract graph data from workspace
    let (nodes, links) = extract_graph_data(workspace);

    // Serialize to JSON for JavaScript consumption
    let nodes_json = serde_json::to_string(&nodes).unwrap_or_else(|_| "[]".to_string());
    let links_json = serde_json::to_string(&links).unwrap_or_else(|_| "[]".to_string());

    let title = format!("Explore - {}", workspace.name);

    // Generate CSS styles
    let extra_styles = generate_explore_styles();

    // Generate JavaScript
    let extra_scripts = generate_explore_scripts(&nodes_json, &links_json);

    // Generate HTML content
    let content = generate_explore_content(nodes.len(), links.len());

    let layout_config = LayoutConfig {
        title: &title,
        workspace_name: Some(&workspace.name),
        workspace_id,
        base_path,
        active_nav: NavItem::Explore,
        content_type: ContentType::ToolbarViewport,
        extra_head: &extra_styles,
        extra_body_end: &extra_scripts,
    };

    generate_page_layout(&layout_config, &content)
}

/// Generates the CSS styles for the explore diagram.
fn generate_explore_styles() -> String {
    r##"<style>
        .explore-toolbar {
            background: var(--toolbar-bg);
            padding: 10px 20px;
            display: flex;
            align-items: center;
            gap: 20px;
            border-bottom: 1px solid var(--toolbar-border);
            flex-shrink: 0;
            z-index: 10;
            position: relative;
        }
        .explore-main {
            display: flex;
            flex: 1;
            overflow: hidden;
        }
        #tree-panel {
            width: 280px;
            min-width: 200px;
            max-width: 500px;
            display: flex;
            flex-direction: column;
            background: var(--bg-secondary);
            border-right: 1px solid var(--border-color);
            flex-shrink: 0;
        }
        #panel-divider {
            width: 5px;
            cursor: col-resize;
            background: var(--border-color);
            transition: background 0.2s;
            flex-shrink: 0;
        }
        #panel-divider:hover,
        #panel-divider.active {
            background: var(--link-color, #3b82f6);
        }
        .tree-header {
            padding: 12px;
            border-bottom: 1px solid var(--border-color);
            flex-shrink: 0;
        }
        .tree-title {
            font-size: 14px;
            font-weight: 600;
            color: var(--text-primary);
            margin-bottom: 8px;
        }
        .tree-header-buttons {
            display: flex;
            gap: 6px;
        }
        .tree-search {
            padding: 8px 12px;
            border-bottom: 1px solid var(--border-color);
            flex-shrink: 0;
        }
        .tree-search input {
            width: 100%;
            padding: 6px 10px;
            border: 1px solid var(--border-color);
            border-radius: 4px;
            background: var(--bg-primary);
            color: var(--text-primary);
            font-size: 12px;
            box-sizing: border-box;
        }
        .tree-search input:focus {
            outline: none;
            border-color: var(--link-color, #3b82f6);
        }
        .type-toggles {
            display: flex;
            padding: 8px 12px;
            gap: 4px;
            border-bottom: 1px solid var(--border-color);
            flex-shrink: 0;
        }
        .type-toggle {
            flex: 1;
            padding: 4px 6px;
            font-size: 11px;
            text-align: center;
            border: 1px solid var(--border-color);
            border-radius: 4px;
            cursor: pointer;
            background: var(--bg-tertiary);
            color: var(--text-secondary);
            transition: all 0.15s;
        }
        .type-toggle:hover {
            background: var(--bg-secondary);
        }
        .type-toggle.active {
            background: var(--link-color, #3b82f6);
            color: white;
            border-color: var(--link-color, #3b82f6);
        }
        .tree-actions {
            display: flex;
            padding: 8px 12px;
            gap: 6px;
            border-bottom: 1px solid var(--border-color);
            flex-shrink: 0;
        }
        .tree-content {
            flex: 1;
            overflow-y: auto;
            padding: 8px 0;
        }
        .filter-tree {
            list-style: none;
            margin: 0;
            padding: 0;
        }
        .filter-tree ul {
            list-style: none;
            margin: 0;
            padding: 0;
        }
        .tree-row {
            display: flex;
            align-items: center;
            gap: 4px;
            padding: 4px 12px;
            cursor: pointer;
            transition: background 0.15s;
            user-select: none;
        }
        .tree-row:hover {
            background: var(--bg-tertiary);
        }
        .tree-row.highlighted {
            background: rgba(59, 130, 246, 0.15);
        }
        .tree-row .toggle {
            width: 16px;
            font-size: 10px;
            text-align: center;
            color: var(--text-muted);
            flex-shrink: 0;
            cursor: pointer;
        }
        .tree-row .toggle:hover {
            color: var(--text-primary);
        }
        .tree-row input[type="checkbox"] {
            margin: 0;
            flex-shrink: 0;
            cursor: pointer;
        }
        .tree-row .name {
            flex: 1;
            font-size: 12px;
            color: var(--text-primary);
            white-space: nowrap;
            overflow: hidden;
            text-overflow: ellipsis;
        }
        .tree-row .name:hover {
            color: var(--link-color, #3b82f6);
        }
        .tree-row .type-badge {
            font-size: 9px;
            padding: 2px 4px;
            border-radius: 2px;
            background: var(--bg-tertiary);
            color: var(--text-muted);
            flex-shrink: 0;
        }
        .tree-row .count-badge {
            font-size: 10px;
            color: var(--text-muted);
            flex-shrink: 0;
        }
        .tree-group-header {
            font-weight: 600;
            color: var(--text-secondary);
            font-size: 11px;
            text-transform: uppercase;
            letter-spacing: 0.5px;
        }
        .tree-row.hidden-by-search {
            display: none;
        }
        .tree-row.hidden-by-type {
            display: none;
        }
        .canvas-container {
            flex: 1;
            position: relative;
            background: var(--canvas-bg);
            overflow: hidden;
        }
        #canvas {
            width: 100%;
            height: 100%;
            cursor: grab;
        }
        #canvas.dragging {
            cursor: grabbing;
        }
        .node {
            cursor: pointer;
        }
        .node circle {
            transition: r 0.15s ease-out, stroke 0.15s, stroke-width 0.15s;
        }
        .node:hover circle {
            r: 35;
        }
        .node.dragging circle {
            r: 35;
            filter: brightness(1.2);
        }
        .node.highlighted circle {
            stroke: #f97316;
            stroke-width: 4;
            filter: drop-shadow(0 0 8px rgba(249, 115, 22, 0.6));
        }
        .node.hidden-node {
            display: none;
        }
        .link {
            fill: none;
            stroke: #999;
            stroke-opacity: 0.6;
            stroke-width: 1.5;
        }
        .link:hover {
            stroke-opacity: 1;
            stroke-width: 2;
        }
        .link.hidden-link {
            display: none;
        }
        .link-label.hidden-link {
            display: none;
        }
        #tooltip {
            position: fixed;
            display: none;
            background: var(--bg-secondary);
            border: 1px solid var(--border-color);
            border-radius: 4px;
            padding: 10px;
            box-shadow: 0 2px 8px var(--shadow);
            z-index: 1000;
            max-width: 300px;
            pointer-events: none;
        }
        #tooltip h4 {
            margin: 0 0 5px 0;
            color: var(--text-primary);
        }
        #tooltip .type {
            color: var(--text-secondary);
            font-size: 12px;
            margin-bottom: 5px;
        }
        #tooltip .desc {
            margin-top: 5px;
            font-size: 13px;
            color: var(--text-primary);
        }
        #tooltip .tech {
            margin-top: 5px;
            font-size: 12px;
            color: var(--text-secondary);
        }
        .info {
            position: absolute;
            bottom: 20px;
            left: 20px;
            background: var(--bg-secondary);
            padding: 10px 15px;
            border-radius: 4px;
            font-size: 13px;
            color: var(--text-secondary);
            display: flex;
            align-items: center;
            gap: 5px;
            border: 1px solid var(--border-color);
        }
        .controls input[type="range"] {
            width: 100px;
        }
        .controls label {
            display: flex;
            align-items: center;
            gap: 5px;
            font-size: 14px;
        }
        .help-text {
            font-size: 12px;
            color: var(--text-muted);
            margin-left: auto;
        }
        .btn {
            background: var(--bg-tertiary);
            border: 1px solid var(--border-color);
            padding: 6px 12px;
            border-radius: 4px;
            cursor: pointer;
            color: var(--text-primary);
            font-size: 13px;
        }
        .btn:hover {
            background: var(--bg-secondary);
        }
        .btn-sm {
            padding: 4px 8px;
            font-size: 11px;
        }
    </style>"##.to_string()
}

/// Generates the JavaScript for the explore diagram.
fn generate_explore_scripts(nodes_json: &str, links_json: &str) -> String {
    format!(r##"<script>
(function() {{
    // Utility function to escape HTML
    function escapeHtml(text) {{
        const map = {{
            '&': '&amp;',
            '<': '&lt;',
            '>': '&gt;',
            '"': '&quot;',
            "'": '&#039;'
        }};
        return text ? text.replace(/[&<>"']/g, m => map[m]) : '';
    }}

    // Wait for DOM ready
    if (document.readyState === 'loading') {{
        document.addEventListener('DOMContentLoaded', init);
    }} else {{
        init();
    }}

    function init() {{
        const nodesMap = {{}};

        // Create SVG and groups
        const svg = document.getElementById('canvas');
        const container = document.querySelector('.canvas-container');
        if (!svg || !container) {{
            console.error('Canvas or container not found');
            return;
        }}

        let width = container.clientWidth;
        let height = container.clientHeight;

        svg.setAttribute('viewBox', `0 0 ${{width}} ${{height}}`);

        const g = document.createElementNS('http://www.w3.org/2000/svg', 'g');
        svg.appendChild(g);

        const linkGroup = document.createElementNS('http://www.w3.org/2000/svg', 'g');
        linkGroup.setAttribute('class', 'links');
        g.appendChild(linkGroup);

        const nodeGroup = document.createElementNS('http://www.w3.org/2000/svg', 'g');
        nodeGroup.setAttribute('class', 'nodes');
        g.appendChild(nodeGroup);

        // Zoom and pan state
        let transform = {{ x: 0, y: 0, k: 1 }};
        let isPanning = false;
        let panStart = {{ x: 0, y: 0 }};

        // Drag state - single global state
        let draggedNode = null;
        let dragStart = {{ x: 0, y: 0 }};

        // Zoom handler
        svg.addEventListener('wheel', (e) => {{
            e.preventDefault();
            const rect = svg.getBoundingClientRect();
            const x = e.clientX - rect.left;
            const y = e.clientY - rect.top;

            const delta = e.deltaY > 0 ? 0.9 : 1.1;
            const newK = Math.max(0.1, Math.min(5, transform.k * delta));

            transform.x = x - (x - transform.x) * (newK / transform.k);
            transform.y = y - (y - transform.y) * (newK / transform.k);
            transform.k = newK;

            updateTransform();
        }});

        // Pan handlers
        svg.addEventListener('mousedown', (e) => {{
            if (e.button === 0 && !e.target.closest('.node')) {{
                isPanning = true;
                panStart = {{ x: e.clientX - transform.x, y: e.clientY - transform.y }};
                svg.classList.add('dragging');
            }}
        }});

        // Global mouse move handler for both panning and dragging
        document.addEventListener('mousemove', (e) => {{
            if (isPanning) {{
                transform.x = e.clientX - panStart.x;
                transform.y = e.clientY - panStart.y;
                updateTransform();
            }} else if (draggedNode) {{
                const rect = svg.getBoundingClientRect();
                // Convert screen coordinates to SVG coordinates
                const svgX = (e.clientX - rect.left - transform.x) / transform.k;
                const svgY = (e.clientY - rect.top - transform.y) / transform.k;

                draggedNode.fx = svgX;
                draggedNode.fy = svgY;
                draggedNode.x = svgX;
                draggedNode.y = svgY;

                // Update the node's position immediately
                updateNodePosition(draggedNode);
                updateLinks();
            }}
        }});

        // Global mouse up handler
        document.addEventListener('mouseup', () => {{
            if (isPanning) {{
                isPanning = false;
                svg.classList.remove('dragging');
            }}
            if (draggedNode) {{
                draggedNode.element.classList.remove('dragging');
                // Release fixed position - let simulation take over
                draggedNode.fx = undefined;
                draggedNode.fy = undefined;
                // Give node some velocity based on recent movement
                simulation.alpha = 0.3;
                requestAnimationFrame(tick);
                draggedNode = null;
            }}
        }});

        function updateTransform() {{
            g.setAttribute('transform', `translate(${{transform.x}},${{transform.y}}) scale(${{transform.k}})`);
        }}

        function updateNodePosition(node) {{
            if (node.element) {{
                node.element.setAttribute('transform', `translate(${{node.x}},${{node.y}})`);
            }}
        }}

        // Node colors by type
        const colors = {{
            'Person': '#08427b',
            'Software System': '#1168bd',
            'Container': '#438dd5',
            'Component': '#85bbf0'
        }};

        // Initialize force simulation with adaptive parameters
        const data = {{
            nodes: {nodes_json},
            links: {links_json}
        }};

        const nodeCount = data.nodes.length;
        const nodeRadius = 30; // Match the circle radius

        // Adaptive parameters based on node count - optimized for less overlap
        const initialSpread = Math.max(600, Math.sqrt(nodeCount) * 120);
        const chargeStrength = Math.min(-200, -500 - nodeCount * 10);
        const linkDist = Math.max(120, 200 - nodeCount * 0.3);
        const collisionRadius = nodeRadius + 15; // Collision buffer around nodes

        let simulation = {{
            nodes: [],
            links: [],
            alpha: 1,
            alphaDecay: 0.008,
            velocityDecay: 0.4,
            chargeStrength: chargeStrength,
            linkDistance: linkDist,
            collisionRadius: collisionRadius,
            collisionStrength: 0.8
        }};

        // Create node elements
        data.nodes.forEach((node, index) => {{
            // Spread nodes in a circle initially for better distribution
            const angle = (index / nodeCount) * 2 * Math.PI;
            const radius = initialSpread * (0.5 + Math.random() * 0.5);
            node.x = width / 2 + Math.cos(angle) * radius;
            node.y = height / 2 + Math.sin(angle) * radius;
            node.vx = 0;
            node.vy = 0;

            const nodeG = document.createElementNS('http://www.w3.org/2000/svg', 'g');
            nodeG.setAttribute('class', 'node');
            nodeG.setAttribute('data-id', node.id);

            const circle = document.createElementNS('http://www.w3.org/2000/svg', 'circle');
            circle.setAttribute('r', 30);
            const nodeColor = colors[node.type] || '#999';
            circle.setAttribute('fill', nodeColor);
            circle.setAttribute('stroke', '#fff');
            circle.setAttribute('stroke-width', 2);

            const text = document.createElementNS('http://www.w3.org/2000/svg', 'text');
            text.setAttribute('text-anchor', 'middle');
            text.setAttribute('dy', '0.35em');
            text.setAttribute('font-size', '11');
            text.setAttribute('fill', 'white');
            text.setAttribute('pointer-events', 'none');
            text.textContent = node.name.length > 12 ? node.name.substring(0, 10) + '...' : node.name;

            nodeG.appendChild(circle);
            nodeG.appendChild(text);
            nodeGroup.appendChild(nodeG);

            node.element = nodeG;
            nodesMap[node.id] = node;

            // Drag handler - start drag on mousedown
            nodeG.addEventListener('mousedown', (e) => {{
                e.stopPropagation();
                e.preventDefault();

                draggedNode = node;
                nodeG.classList.add('dragging');

                // Fix the node position
                node.fx = node.x;
                node.fy = node.y;
            }});

            // Tooltip handlers
            nodeG.addEventListener('mouseenter', (e) => {{
                if (draggedNode) return; // Don't show tooltip while dragging
                const tooltip = document.getElementById('tooltip');
                let html = `<h4>${{escapeHtml(node.name)}}</h4><div class="type">${{node.type}}</div>`;
                if (node.description) {{
                    html += `<div class="desc">${{escapeHtml(node.description)}}</div>`;
                }}
                if (node.technology) {{
                    html += `<div class="tech">Technology: ${{escapeHtml(node.technology)}}</div>`;
                }}
                tooltip.innerHTML = html;
                tooltip.style.display = 'block';
            }});

            nodeG.addEventListener('mousemove', (e) => {{
                if (draggedNode) return;
                const tooltip = document.getElementById('tooltip');
                tooltip.style.left = (e.clientX + 15) + 'px';
                tooltip.style.top = (e.clientY + 15) + 'px';
            }});

            nodeG.addEventListener('mouseleave', () => {{
                document.getElementById('tooltip').style.display = 'none';
            }});
        }});

        // Create link elements
        const linkElements = [];
        data.links.forEach(link => {{
            const source = nodesMap[link.source];
            const target = nodesMap[link.target];

            if (!source || !target) return;

            link.source = source;
            link.target = target;

            const path = document.createElementNS('http://www.w3.org/2000/svg', 'path');
            path.setAttribute('class', 'link');
            linkGroup.appendChild(path);

            link.element = path;
            linkElements.push(link);

            if (link.label) {{
                const text = document.createElementNS('http://www.w3.org/2000/svg', 'text');
                text.setAttribute('class', 'link-label');
                text.setAttribute('font-size', '9');
                text.setAttribute('fill', '#666');
                text.setAttribute('pointer-events', 'none');
                const truncatedLabel = link.label.length > 15 ? link.label.substring(0, 13) + '...' : link.label;
                text.textContent = truncatedLabel;
                linkGroup.appendChild(text);
                link.labelElement = text;
            }}
        }});

        simulation.nodes = data.nodes;
        simulation.links = linkElements;

        // Update link positions
        function updateLinks() {{
            simulation.links.forEach(link => {{
                const sx = link.source.x;
                const sy = link.source.y;
                const tx = link.target.x;
                const ty = link.target.y;

                const dx = tx - sx;
                const dy = ty - sy;
                const dist = Math.sqrt(dx * dx + dy * dy) || 1;

                // Calculate control point for curved line
                const midX = (sx + tx) / 2;
                const midY = (sy + ty) / 2;
                const offset = Math.min(30, dist * 0.15);
                const perpX = -dy / dist * offset;
                const perpY = dx / dist * offset;

                link.element.setAttribute('d',
                    `M${{sx}},${{sy}} Q${{midX + perpX}},${{midY + perpY}} ${{tx}},${{ty}}`);

                if (link.labelElement) {{
                    link.labelElement.setAttribute('x', midX + perpX);
                    link.labelElement.setAttribute('y', midY + perpY);
                    link.labelElement.setAttribute('text-anchor', 'middle');
                }}
            }});
        }}

        // Force simulation
        function applyForces() {{
            const alpha = simulation.alpha;
            const nodes = simulation.nodes;
            const links = simulation.links;
            const collisionRadius = simulation.collisionRadius;
            const collisionStrength = simulation.collisionStrength;

            // Pass 1: Apply charge force (repulsion between all nodes)
            for (let i = 0; i < nodes.length; i++) {{
                const node1 = nodes[i];

                for (let j = i + 1; j < nodes.length; j++) {{
                    const node2 = nodes[j];
                    let dx = node2.x - node1.x;
                    let dy = node2.y - node1.y;
                    let dist = Math.sqrt(dx * dx + dy * dy);

                    // Prevent division by zero with jitter
                    if (dist < 1) {{
                        dx = (Math.random() - 0.5) * 2;
                        dy = (Math.random() - 0.5) * 2;
                        dist = Math.sqrt(dx * dx + dy * dy);
                    }}

                    // Stronger repulsion at close range (inverse square law)
                    const strength = simulation.chargeStrength * alpha / (dist * dist);
                    const fx = (dx / dist) * strength;
                    const fy = (dy / dist) * strength;

                    if (node1.fx === undefined) {{
                        node1.vx -= fx;
                        node1.vy -= fy;
                    }}
                    if (node2.fx === undefined) {{
                        node2.vx += fx;
                        node2.vy += fy;
                    }}
                }}
            }}

            // Pass 2: Apply collision force (hard constraint to prevent overlaps)
            for (let i = 0; i < nodes.length; i++) {{
                const node1 = nodes[i];

                for (let j = i + 1; j < nodes.length; j++) {{
                    const node2 = nodes[j];
                    let dx = node2.x - node1.x;
                    let dy = node2.y - node1.y;
                    let dist = Math.sqrt(dx * dx + dy * dy);

                    const minDist = collisionRadius * 2; // Both nodes have collision radius

                    if (dist < minDist) {{
                        // Nodes are overlapping - push them apart
                        if (dist < 1) {{
                            // Nodes are at same position - add random jitter
                            dx = (Math.random() - 0.5) * 2;
                            dy = (Math.random() - 0.5) * 2;
                            dist = Math.sqrt(dx * dx + dy * dy);
                        }}

                        const overlap = minDist - dist;
                        const force = overlap * collisionStrength * 0.5;
                        const fx = (dx / dist) * force;
                        const fy = (dy / dist) * force;

                        if (node1.fx === undefined) {{
                            node1.x -= fx;
                            node1.y -= fy;
                        }}
                        if (node2.fx === undefined) {{
                            node2.x += fx;
                            node2.y += fy;
                        }}
                    }}
                }}
            }}

            // Pass 3: Apply link force (spring attraction between connected nodes)
            links.forEach(link => {{
                const dx = link.target.x - link.source.x;
                const dy = link.target.y - link.source.y;
                const dist = Math.sqrt(dx * dx + dy * dy) || 1;

                // Spring force - pulls towards ideal distance
                const displacement = dist - simulation.linkDistance;
                const force = displacement / dist * alpha * 0.15;

                const fx = dx * force;
                const fy = dy * force;

                if (link.source.fx === undefined) {{
                    link.source.vx += fx;
                    link.source.vy += fy;
                }}
                if (link.target.fx === undefined) {{
                    link.target.vx -= fx;
                    link.target.vy -= fy;
                }}
            }});

            // Pass 4: Apply gentle center force (keeps graph from drifting)
            const centerStrength = 0.003 * alpha;
            nodes.forEach(node => {{
                if (node.fx !== undefined) return;
                node.vx += (width / 2 - node.x) * centerStrength;
                node.vy += (height / 2 - node.y) * centerStrength;
            }});

            // Pass 5: Apply velocity and update positions
            nodes.forEach(node => {{
                if (node.fx !== undefined) {{
                    node.x = node.fx;
                    node.y = node.fy;
                }} else {{
                    // Apply velocity decay (friction)
                    node.vx *= (1 - simulation.velocityDecay);
                    node.vy *= (1 - simulation.velocityDecay);

                    // Clamp velocity to prevent instability
                    const maxVel = 50;
                    const vel = Math.sqrt(node.vx * node.vx + node.vy * node.vy);
                    if (vel > maxVel) {{
                        node.vx = (node.vx / vel) * maxVel;
                        node.vy = (node.vy / vel) * maxVel;
                    }}

                    node.x += node.vx;
                    node.y += node.vy;
                }}
            }});
        }}

        // Animation loop
        function tick() {{
            if (simulation.alpha < 0.001) return;

            simulation.alpha *= (1 - simulation.alphaDecay);

            applyForces();

            // Update node positions
            simulation.nodes.forEach(updateNodePosition);

            // Update link positions
            updateLinks();

            requestAnimationFrame(tick);
        }}

        // Start simulation
        tick();

        // Control handlers
        document.getElementById('resetBtn').addEventListener('click', () => {{
            simulation.nodes.forEach((node, index) => {{
                const angle = (index / nodeCount) * 2 * Math.PI;
                const radius = initialSpread * (0.5 + Math.random() * 0.5);
                node.x = width / 2 + Math.cos(angle) * radius;
                node.y = height / 2 + Math.sin(angle) * radius;
                node.vx = 0;
                node.vy = 0;
                node.fx = undefined;
                node.fy = undefined;
            }});
            simulation.alpha = 1;
            tick();
        }});

        document.getElementById('centerBtn').addEventListener('click', () => {{
            transform = {{ x: 0, y: 0, k: 1 }};
            updateTransform();
        }});

        document.getElementById('chargeSlider').addEventListener('input', (e) => {{
            simulation.chargeStrength = -parseFloat(e.target.value);
            simulation.alpha = Math.max(0.3, simulation.alpha);
            if (simulation.alpha >= 0.001) tick();
        }});

        document.getElementById('linkDistanceSlider').addEventListener('input', (e) => {{
            simulation.linkDistance = parseFloat(e.target.value);
            simulation.alpha = Math.max(0.3, simulation.alpha);
            if (simulation.alpha >= 0.001) tick();
        }});

        document.getElementById('collisionSlider').addEventListener('input', (e) => {{
            simulation.collisionRadius = parseFloat(e.target.value);
            simulation.alpha = Math.max(0.5, simulation.alpha);
            if (simulation.alpha >= 0.001) tick();
        }});

        // ========================================
        // Tree Filter Panel Functionality
        // ========================================

        // Get workspace ID for localStorage key
        const workspaceId = document.body.dataset.workspaceId || window.location.pathname.split('/w/')[1]?.split('/')[0] || 'default';
        const storageKey = `structurizr-explore-filter-${{workspaceId}}`;

        // Filter state management
        const filterState = {{
            visibility: {{}},
            expandedNodes: new Set(['__people__', '__systems__']),
            panelWidth: 280,
            hiddenTypes: new Set(),

            init(nodes) {{
                // Initialize all nodes as visible
                nodes.forEach(node => {{
                    this.visibility[node.id] = true;
                }});
                // Load saved state if exists
                this.load();
            }},

            setVisible(id, visible, cascade = true) {{
                this.visibility[id] = visible;

                if (cascade) {{
                    // Find all descendants and set same visibility
                    const descendants = getDescendants(id);
                    descendants.forEach(childId => {{
                        this.visibility[childId] = visible;
                    }});
                }}

                // Update parent checkbox states
                updateParentStates(id);
                this.save();
            }},

            isVisible(id) {{
                return this.visibility[id] !== false;
            }},

            setAllVisible(visible) {{
                Object.keys(this.visibility).forEach(id => {{
                    this.visibility[id] = visible;
                }});
                this.save();
            }},

            save() {{
                try {{
                    localStorage.setItem(storageKey, JSON.stringify({{
                        visibility: this.visibility,
                        panelWidth: this.panelWidth,
                        expandedNodes: Array.from(this.expandedNodes)
                    }}));
                }} catch (e) {{
                    console.warn('Failed to save filter state:', e);
                }}
            }},

            load() {{
                try {{
                    const stored = localStorage.getItem(storageKey);
                    if (stored) {{
                        const data = JSON.parse(stored);
                        if (data.visibility) this.visibility = data.visibility;
                        if (data.panelWidth) this.panelWidth = data.panelWidth;
                        if (data.expandedNodes) this.expandedNodes = new Set(data.expandedNodes);
                        return true;
                    }}
                }} catch (e) {{
                    console.warn('Failed to load filter state:', e);
                }}
                return false;
            }}
        }};

        // Helper: Get all descendant IDs of a node
        function getDescendants(nodeId) {{
            const descendants = [];
            const queue = [nodeId];

            while (queue.length > 0) {{
                const currentId = queue.shift();
                // Find children (nodes whose parent_id matches currentId)
                data.nodes.forEach(node => {{
                    if (node.parent_id === currentId) {{
                        descendants.push(node.id);
                        queue.push(node.id);
                    }}
                }});
            }}

            return descendants;
        }}

        // Helper: Get parent ID of a node
        function getParentId(nodeId) {{
            const node = data.nodes.find(n => n.id === nodeId);
            return node?.parent_id || null;
        }}

        // Helper: Get children of a node
        function getChildren(nodeId) {{
            if (nodeId === '__people__') {{
                return data.nodes.filter(n => n.type === 'Person');
            }}
            if (nodeId === '__systems__') {{
                return data.nodes.filter(n => n.type === 'Software System');
            }}
            return data.nodes.filter(n => n.parent_id === nodeId);
        }}

        // Update parent checkbox indeterminate states
        function updateParentStates(changedId) {{
            let parentId = getParentId(changedId);

            // Also check group parents
            const node = data.nodes.find(n => n.id === changedId);
            if (node) {{
                if (node.type === 'Person') parentId = '__people__';
                else if (node.type === 'Software System') parentId = '__systems__';
            }}

            while (parentId) {{
                const children = getChildren(parentId);
                if (children.length === 0) break;

                const allChecked = children.every(c => filterState.visibility[c.id] !== false);
                const noneChecked = children.every(c => filterState.visibility[c.id] === false);

                const checkbox = document.querySelector(`[data-node-id="${{parentId}}"] input[type="checkbox"]`);
                if (checkbox) {{
                    checkbox.checked = allChecked;
                    checkbox.indeterminate = !allChecked && !noneChecked;
                }}

                // Move up the tree
                if (parentId === '__people__' || parentId === '__systems__') break;
                parentId = getParentId(parentId);
            }}
        }}

        // Build hierarchical tree structure from flat nodes
        function buildTree(nodes) {{
            const people = nodes.filter(n => n.type === 'Person');
            const systems = nodes.filter(n => n.type === 'Software System');

            const systemTrees = systems.map(system => {{
                const containers = nodes.filter(n =>
                    n.type === 'Container' && n.parent_id === system.id
                );

                const containerTrees = containers.map(container => {{
                    const components = nodes.filter(n =>
                        n.type === 'Component' && n.parent_id === container.id
                    );
                    return {{ ...container, children: components }};
                }});

                return {{ ...system, children: containerTrees }};
            }});

            return [
                {{ id: '__people__', name: 'People', type: 'group', children: people }},
                {{ id: '__systems__', name: 'Systems', type: 'group', children: systemTrees }}
            ];
        }}

        // Render tree in the tree content area
        function renderTree(treeData) {{
            const container = document.getElementById('treeContent');
            if (!container) return;

            container.innerHTML = '';

            function renderNode(node, depth = 0) {{
                const li = document.createElement('li');
                li.dataset.nodeId = node.id;
                li.dataset.nodeType = node.type;

                const row = document.createElement('div');
                row.className = 'tree-row';
                row.style.paddingLeft = `${{depth * 16 + 8}}px`;

                // Toggle for expandable nodes
                const hasChildren = node.children && node.children.length > 0;
                const toggle = document.createElement('span');
                toggle.className = 'toggle';
                if (hasChildren) {{
                    const isExpanded = filterState.expandedNodes.has(node.id);
                    toggle.textContent = isExpanded ? '▼' : '▶';
                    toggle.addEventListener('click', (e) => {{
                        e.stopPropagation();
                        toggleExpand(node.id);
                    }});
                }}
                row.appendChild(toggle);

                // Checkbox
                const checkbox = document.createElement('input');
                checkbox.type = 'checkbox';
                if (node.type === 'group') {{
                    // Group checkbox - check based on children
                    const children = node.children || [];
                    const allChecked = children.every(c => filterState.visibility[c.id] !== false);
                    const noneChecked = children.every(c => filterState.visibility[c.id] === false);
                    checkbox.checked = allChecked;
                    checkbox.indeterminate = !allChecked && !noneChecked;
                }} else {{
                    checkbox.checked = filterState.isVisible(node.id);
                }}
                checkbox.addEventListener('change', (e) => {{
                    e.stopPropagation();
                    if (node.type === 'group') {{
                        // Toggle all children in group
                        const children = node.children || [];
                        children.forEach(child => {{
                            filterState.setVisible(child.id, e.target.checked, true);
                        }});
                    }} else {{
                        filterState.setVisible(node.id, e.target.checked, true);
                    }}
                    updateGraph();
                    renderTree(buildTree(data.nodes));
                }});
                row.appendChild(checkbox);

                // Name (clickable to focus in graph)
                const name = document.createElement('span');
                name.className = 'name' + (node.type === 'group' ? ' tree-group-header' : '');
                name.textContent = node.name;
                if (node.type !== 'group') {{
                    name.addEventListener('click', (e) => {{
                        e.stopPropagation();
                        focusNode(node.id);
                    }});
                }}
                row.appendChild(name);

                // Child count for groups/parents
                if (hasChildren) {{
                    const count = document.createElement('span');
                    count.className = 'count-badge';
                    const visibleCount = node.children.filter(c =>
                        c.type === 'group' || filterState.visibility[c.id] !== false
                    ).length;
                    count.textContent = `(${{visibleCount}}/${{node.children.length}})`;
                    row.appendChild(count);
                }}

                li.appendChild(row);

                // Children
                if (hasChildren) {{
                    const ul = document.createElement('ul');
                    ul.style.display = filterState.expandedNodes.has(node.id) ? 'block' : 'none';
                    node.children.forEach(child => {{
                        ul.appendChild(renderNode(child, depth + 1));
                    }});
                    li.appendChild(ul);
                }}

                return li;
            }}

            const ul = document.createElement('ul');
            ul.className = 'filter-tree';
            treeData.forEach(root => {{
                ul.appendChild(renderNode(root, 0));
            }});
            container.appendChild(ul);
        }}

        // Toggle tree node expansion
        function toggleExpand(nodeId) {{
            if (filterState.expandedNodes.has(nodeId)) {{
                filterState.expandedNodes.delete(nodeId);
            }} else {{
                filterState.expandedNodes.add(nodeId);
            }}
            filterState.save();
            renderTree(buildTree(data.nodes));
        }}

        // Update graph based on filter state
        function updateGraph() {{
            let visibleNodeCount = 0;
            let visibleLinkCount = 0;

            // Update node visibility
            simulation.nodes.forEach(node => {{
                const visible = filterState.visibility[node.id] !== false;
                node._filtered = !visible;

                if (node.element) {{
                    if (visible) {{
                        node.element.classList.remove('hidden-node');
                        visibleNodeCount++;
                    }} else {{
                        node.element.classList.add('hidden-node');
                    }}
                }}
            }});

            // Update link visibility - hide if either endpoint is hidden
            simulation.links.forEach(link => {{
                const sourceVisible = filterState.visibility[link.source.id] !== false;
                const targetVisible = filterState.visibility[link.target.id] !== false;
                const visible = sourceVisible && targetVisible;

                if (link.element) {{
                    if (visible) {{
                        link.element.classList.remove('hidden-link');
                        visibleLinkCount++;
                    }} else {{
                        link.element.classList.add('hidden-link');
                    }}
                }}
                if (link.labelElement) {{
                    if (visible) {{
                        link.labelElement.classList.remove('hidden-link');
                    }} else {{
                        link.labelElement.classList.add('hidden-link');
                    }}
                }}
            }});

            // Update info panel
            const nodeCountEl = document.getElementById('nodeCount');
            const linkCountEl = document.getElementById('linkCount');
            if (nodeCountEl) nodeCountEl.textContent = `${{visibleNodeCount}}/${{simulation.nodes.length}}`;
            if (linkCountEl) linkCountEl.textContent = `${{visibleLinkCount}}/${{simulation.links.length}}`;
        }}

        // Focus on a node in the graph (pan to it and highlight)
        function focusNode(nodeId) {{
            const node = simulation.nodes.find(n => n.id === nodeId);
            if (!node || node._filtered) return;

            // Calculate transform to center this node
            const canvasContainer = document.querySelector('.canvas-container');
            const centerX = canvasContainer.clientWidth / 2;
            const centerY = canvasContainer.clientHeight / 2;

            const targetX = centerX - node.x * transform.k;
            const targetY = centerY - node.y * transform.k;

            // Animate to position
            const startX = transform.x;
            const startY = transform.y;
            const duration = 300;
            const startTime = performance.now();

            function animate(time) {{
                const elapsed = time - startTime;
                const t = Math.min(elapsed / duration, 1);
                const eased = 1 - Math.pow(1 - t, 3); // ease-out-cubic

                transform.x = startX + (targetX - startX) * eased;
                transform.y = startY + (targetY - startY) * eased;
                updateTransform();

                if (t < 1) {{
                    requestAnimationFrame(animate);
                }} else {{
                    highlightNode(node);
                }}
            }}
            requestAnimationFrame(animate);
        }}

        // Temporarily highlight a node
        function highlightNode(node) {{
            if (!node.element) return;
            node.element.classList.add('highlighted');
            setTimeout(() => {{
                node.element.classList.remove('highlighted');
            }}, 1500);
        }}

        // Setup resizable panel
        function setupResizablePanel() {{
            const panel = document.getElementById('tree-panel');
            const divider = document.getElementById('panel-divider');
            const canvasContainer = document.querySelector('.canvas-container');

            if (!panel || !divider) return;

            // Apply saved width
            panel.style.width = `${{filterState.panelWidth}}px`;

            let isResizing = false;
            let startX, startWidth;

            divider.addEventListener('mousedown', (e) => {{
                isResizing = true;
                startX = e.clientX;
                startWidth = panel.offsetWidth;
                divider.classList.add('active');
                document.body.style.cursor = 'col-resize';
                document.body.style.userSelect = 'none';
                e.preventDefault();
            }});

            document.addEventListener('mousemove', (e) => {{
                if (!isResizing) return;

                const delta = e.clientX - startX;
                const newWidth = Math.max(200, Math.min(500, startWidth + delta));
                panel.style.width = `${{newWidth}}px`;
            }});

            document.addEventListener('mouseup', () => {{
                if (isResizing) {{
                    isResizing = false;
                    divider.classList.remove('active');
                    document.body.style.cursor = '';
                    document.body.style.userSelect = '';

                    filterState.panelWidth = panel.offsetWidth;
                    filterState.save();

                    // Trigger graph resize
                    width = canvasContainer.clientWidth;
                    height = canvasContainer.clientHeight;
                    svg.setAttribute('viewBox', `0 0 ${{width}} ${{height}}`);
                }}
            }});
        }}

        // Setup search functionality
        function setupSearch() {{
            const searchInput = document.getElementById('treeSearchInput');
            if (!searchInput) return;

            let debounceTimer;
            searchInput.addEventListener('input', (e) => {{
                clearTimeout(debounceTimer);
                debounceTimer = setTimeout(() => {{
                    const query = e.target.value.toLowerCase().trim();
                    filterTreeBySearch(query);
                }}, 150);
            }});
        }}

        function filterTreeBySearch(query) {{
            const treeContent = document.getElementById('treeContent');
            if (!treeContent) return;

            const rows = treeContent.querySelectorAll('.tree-row');
            rows.forEach(row => {{
                const li = row.parentElement;
                const name = row.querySelector('.name')?.textContent.toLowerCase() || '';
                const isGroup = li.dataset.nodeType === 'group';

                if (query === '' || name.includes(query) || isGroup) {{
                    row.classList.remove('hidden-by-search');
                    // If matching, expand parents to show it
                    if (query !== '' && !isGroup && name.includes(query)) {{
                        let parent = li.parentElement;
                        while (parent && parent.tagName === 'UL') {{
                            parent.style.display = 'block';
                            parent = parent.parentElement?.parentElement;
                        }}
                    }}
                }} else {{
                    row.classList.add('hidden-by-search');
                }}
            }});
        }}

        // Setup type toggles
        function setupTypeToggles() {{
            const toggles = document.querySelectorAll('.type-toggle');
            toggles.forEach(toggle => {{
                toggle.addEventListener('click', () => {{
                    const type = toggle.dataset.type;
                    toggle.classList.toggle('active');
                    const isActive = toggle.classList.contains('active');

                    // Show/hide all nodes of this type
                    data.nodes.forEach(node => {{
                        if (node.type === type) {{
                            filterState.setVisible(node.id, isActive, true);
                        }}
                    }});

                    updateGraph();
                    renderTree(buildTree(data.nodes));
                }});
            }});
        }}

        // Setup expand/collapse all
        function setupExpandCollapse() {{
            const expandBtn = document.getElementById('expandAllBtn');
            const collapseBtn = document.getElementById('collapseAllBtn');

            if (expandBtn) {{
                expandBtn.addEventListener('click', () => {{
                    // Expand all nodes that have children
                    filterState.expandedNodes.add('__people__');
                    filterState.expandedNodes.add('__systems__');
                    data.nodes.forEach(node => {{
                        const hasChildren = data.nodes.some(n => n.parent_id === node.id);
                        if (hasChildren) {{
                            filterState.expandedNodes.add(node.id);
                        }}
                    }});
                    filterState.save();
                    renderTree(buildTree(data.nodes));
                }});
            }}

            if (collapseBtn) {{
                collapseBtn.addEventListener('click', () => {{
                    filterState.expandedNodes.clear();
                    filterState.expandedNodes.add('__people__');
                    filterState.expandedNodes.add('__systems__');
                    filterState.save();
                    renderTree(buildTree(data.nodes));
                }});
            }}
        }}

        // Setup enable/disable all buttons
        function setupEnableDisableAll() {{
            const enableBtn = document.getElementById('enableAllBtn');
            const disableBtn = document.getElementById('disableAllBtn');

            if (enableBtn) {{
                enableBtn.addEventListener('click', () => {{
                    filterState.setAllVisible(true);
                    updateGraph();
                    renderTree(buildTree(data.nodes));
                }});
            }}

            if (disableBtn) {{
                disableBtn.addEventListener('click', () => {{
                    filterState.setAllVisible(false);
                    updateGraph();
                    renderTree(buildTree(data.nodes));
                }});
            }}
        }}

        // Initialize tree panel
        filterState.init(data.nodes);
        renderTree(buildTree(data.nodes));
        setupResizablePanel();
        setupSearch();
        setupTypeToggles();
        setupExpandCollapse();
        setupEnableDisableAll();

        // Apply initial filter state from localStorage
        updateGraph();

        // Window resize handler
        window.addEventListener('resize', () => {{
            const canvasContainer = document.querySelector('.canvas-container');
            width = canvasContainer.clientWidth;
            height = canvasContainer.clientHeight;
            svg.setAttribute('viewBox', `0 0 ${{width}} ${{height}}`);
        }});
    }}
}})();
</script>"##)
}

/// Generates the HTML content for the explore diagram.
fn generate_explore_content(node_count: usize, link_count: usize) -> String {
    format!(r##"
        <div class="explore-toolbar">
            <button id="resetBtn" class="btn">Reset</button>
            <button id="centerBtn" class="btn">Center</button>

            <div class="controls">
                <label>
                    Repulsion:
                    <input type="range" id="chargeSlider" min="200" max="2000" value="800" step="100">
                </label>
            </div>

            <div class="controls">
                <label>
                    Link Distance:
                    <input type="range" id="linkDistanceSlider" min="80" max="400" value="180" step="20">
                </label>
            </div>

            <div class="controls">
                <label>
                    Collision:
                    <input type="range" id="collisionSlider" min="30" max="80" value="45" step="5">
                </label>
            </div>

            <span class="help-text">
                Drag nodes to move • Pan with mouse • Scroll to zoom
            </span>
        </div>

        <div class="explore-main">
            <div id="tree-panel">
                <div class="tree-header">
                    <div class="tree-title">Elements</div>
                    <div class="tree-header-buttons">
                        <button id="enableAllBtn" class="btn btn-sm">Enable All</button>
                        <button id="disableAllBtn" class="btn btn-sm">Disable All</button>
                    </div>
                </div>
                <div class="tree-search">
                    <input type="text" id="treeSearchInput" placeholder="Search elements...">
                </div>
                <div class="type-toggles">
                    <button class="type-toggle active" data-type="Person" title="People">P</button>
                    <button class="type-toggle active" data-type="Software System" title="Software Systems">S</button>
                    <button class="type-toggle active" data-type="Container" title="Containers">C</button>
                    <button class="type-toggle active" data-type="Component" title="Components">Co</button>
                </div>
                <div class="tree-actions">
                    <button id="expandAllBtn" class="btn btn-sm">Expand All</button>
                    <button id="collapseAllBtn" class="btn btn-sm">Collapse All</button>
                </div>
                <div class="tree-content" id="treeContent"></div>
            </div>
            <div id="panel-divider"></div>
            <div class="canvas-container">
                <svg id="canvas"></svg>
                <div class="info">
                    <span id="nodeCount">{}</span> nodes &bull; <span id="linkCount">{}</span> links
                </div>
            </div>
        </div>

        <div id="tooltip"></div>
    "##, node_count, link_count)
}
