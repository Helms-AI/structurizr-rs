# ADR-002: Task Assignment and Workflow Engine Design

## Status
Accepted

## Context
FreshMart stores generate over 500,000 tasks daily including inventory restocking, equipment maintenance, customer service requests, and compliance checks. We need a task management system that:
- Assigns tasks to the optimal associate based on skills, location, and workload
- Prioritizes tasks dynamically based on urgency and business impact
- Tracks task lifecycle from creation through completion
- Integrates with IoT alerts for automated task generation
- Supports complex workflows with conditional branching and parallel execution

## Decision
We will implement a task management system using Camunda as the workflow engine with ML-powered optimization:

1. **Task Engine (Camunda)**: BPMN-based workflow orchestration for task lifecycle management
2. **Priority Engine (ML Model)**: Gradient boosting model to calculate dynamic task priority scores
3. **Assignment Engine (OR-Tools)**: Constraint satisfaction solver for optimal task-associate matching

The workflow design will:
- Accept tasks from multiple sources (dashboard, mobile app, IoT alerts, scheduled jobs)
- Calculate priority scores using features: task type, SLA deadline, store traffic, associate availability
- Solve assignment optimization every 30 seconds using OR-Tools CP-SAT solver
- Publish task events to Kafka for real-time tracking
- Support task escalation, reassignment, and delegation

Priority scoring factors:
- Base priority by task category (safety: 100, compliance: 80, customer: 70, inventory: 50)
- Time decay function increasing urgency as deadline approaches
- Store traffic multiplier during peak hours
- Impact score based on revenue/safety implications

## Consequences

### Positive
- 40% reduction in task completion time through optimal assignment
- Consistent SLA adherence with priority-based scheduling
- Flexible workflows supporting diverse task types
- Audit trail and compliance reporting via Camunda cockpit
- Automatic workload balancing across associates

### Negative
- Camunda licensing costs for enterprise features
- ML model requires ongoing training and monitoring
- Complex debugging of optimization decisions
- Latency impact from optimization solver execution

### Mitigation
- Use Camunda Community Edition where enterprise features not needed
- Implement feature store for model input consistency
- Add explainability logging for assignment decisions
- Cache optimal assignments with 30-second TTL

## Implementation
1. Deploy Camunda on Kubernetes with PostgreSQL backend
2. Define BPMN processes for each task category
3. Train priority model on 12 months of historical task data
4. Implement OR-Tools solver with configurable constraints
5. Create REST APIs for task operations
6. Build real-time dashboard for task monitoring
7. Set up alerts for SLA violations and backlog growth

## References
- [Camunda Best Practices](https://camunda.com/best-practices)
- [OR-Tools CP-SAT Solver](https://developers.google.com/optimization/cp/cp_solver)
- [Task Priority Model Training](https://wiki.freshmart.com/ml-task-priority)
