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
            transition: r 0.15s ease-out;
        }
        .node:hover circle {
            r: 35;
        }
        .node.dragging circle {
            r: 35;
            filter: brightness(1.2);
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

        // Window resize handler
        window.addEventListener('resize', () => {{
            width = container.clientWidth;
            height = container.clientHeight;
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

        <div class="canvas-container">
            <svg id="canvas"></svg>
            <div class="info">
                {} nodes &bull; {} links
            </div>
        </div>

        <div id="tooltip"></div>
    "##, node_count, link_count)
}
