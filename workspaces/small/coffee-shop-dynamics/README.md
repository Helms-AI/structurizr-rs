# Coffee Shop Ordering System

A simple coffee shop ordering system demonstrating **dynamic diagrams** in Structurizr DSL.

## Overview

This example models a small artisan coffee shop with:
- Touch-screen POS terminals for customer ordering
- Real-time order queue display for baristas
- Inventory tracking to ensure ingredient availability

## Dynamic Diagram Feature

The key feature of this example is the **OrderFlow** dynamic diagram that shows the step-by-step sequence of placing a coffee order:

1. Customer selects drink and customizations
2. POS submits order to the queue service
3. Queue service checks ingredient availability
4. Inventory service reads stock levels
5. Inventory confirms availability
6. Order is saved with pending status
7. Order appears on barista's queue screen

## Running the Example

```bash
./serve.sh
```

Then open the URL shown in your browser and navigate to the **OrderFlow** dynamic view.

## Architecture Highlights

- **Simple 4-container architecture** - Easy to understand
- **Clear separation of concerns** - POS, Queue, Inventory, Database
- **Real-time workflow** - Dynamic diagram shows temporal sequence

## DSL Features Demonstrated

- Dynamic views with step-by-step interactions
- Container diagrams with custom shapes
- Relationship descriptions with technologies
- Tag-based styling
