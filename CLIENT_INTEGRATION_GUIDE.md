# Client-Side Integration Guide: Streaming Pool Updates

This guide provides a comprehensive walkthrough for connecting a Next.js frontend to the Heimdall real-time streaming service.

## 1. Overview

The `heimdall/stream` service provides live, pool-specific event data (such as swaps or balance changes) over a gRPC-web connection. Clients subscribe to a specific `pool_id` and receive a stream of events only for that pool.

## 2. Prerequisites

Before you can connect your client, you need to generate the necessary JavaScript/TypeScript client code from the service's `.proto` definition.

### Dependencies

Install the required client libraries:

```bash
npm install grpc-web google-protobuf
```

### Code Generation

From the root of the `heimdall` monorepo, run the `protoc` compiler. This command reads the `stream.proto` file and generates a `StreamServiceClientPb.ts` (the client) and `stream_pb.js` (the messages).

```bash
# Adjust the output path to your frontend's preferred location
# e.g., 'my-next-app/generated/grpc'
OUTPUT_DIR="./my-next-app/generated/grpc"

# Create the directory if it doesn't exist
mkdir -p $OUTPUT_DIR

# Run the protoc compiler
protoc -I=packages/stream/proto \
  packages/stream/proto/stream.proto \
  --js_out=import_style=commonjs,binary:$OUTPUT_DIR \
  --grpc-web_out=import_style=typescript,mode=grpcwebtext:$OUTPUT_DIR
```

This generates the necessary files for you to interact with the gRPC service in a type-safe way.

## 3. The `PoolUpdate` Message

All events for all pool types are sent as a `PoolUpdate` message. This provides a consistent message structure for the client.

```proto
message PoolUpdate {
    string pool_id = 1;      // The ID of the pool this event belongs to.
    string event_type = 2;   // The type of event, e.g., "dbc_swap", "dbc_balance_update".
    string payload_json = 3; // A JSON string containing the specific data for the event.
}
```

-   **`pool_id`**: Use this to confirm the update is for the pool you requested.
-   **`event_type`**: Use this to determine how to parse and display the `payload_json`.
-   **`payload_json`**: The core data. You will need to `JSON.parse()` this string to access the detailed event fields (e.g., `signature`, `input_amount`).

## 4. React Component Implementation

The following is a robust, reusable React component (`PoolStreamViewer.tsx`) for subscribing to and displaying pool updates.

### Key Concepts:

-   **Client-Side Execution**: The gRPC client must be instantiated inside `useEffect` to ensure it only runs in the browser, not during server-side rendering.
-   **State Management**: It uses `useState` to store the list of incoming events, connection status, and any errors.
-   **Stream Lifecycle**: The `useEffect` hook correctly manages the stream's lifecycle. It initiates the connection when the component mounts and, critically, calls `stream.cancel()` in its cleanup function when the component unmounts to prevent memory leaks.

### `components/PoolStreamViewer.tsx`

```typescript
import React, { useEffect, useState } from 'react';

// Adjust the import path to where you generated the gRPC files
import { HeimdallStreamClient } from '../generated/grpc/StreamServiceClientPb';
import { PoolUpdateRequest, PoolUpdate } from '../generated/grpc/stream_pb';

// Define a type for our structured, displayable event data
interface DisplayEvent {
  poolId: string;
  eventType: string;
  payload: any; // The parsed JSON payload
  receivedAt: string;
}

const PoolStreamViewer = ({ poolId }: { poolId: string }) => {
  const [events, setEvents] = useState<DisplayEvent[]>([]);
  const [error, setError] = useState<string | null>(null);
  const [status, setStatus] = useState<string>('Idle');

  useEffect(() => {
    // Ensure this code only runs on the client
    if (typeof window === 'undefined' || !poolId) {
      return;
    }

    // Must be instantiated client-side
    const client = new HeimdallStreamClient('http://localhost:50051');

    const request = new PoolUpdateRequest();
    request.setPoolId(poolId);

    console.log(`Subscribing to pool: ${poolId}`);
    setStatus('Connecting...');

    const stream = client.streamPoolUpdates(request, {});

    stream.on('data', (response: PoolUpdate) => {
      if (!status.includes('Receiving')) setStatus('Receiving data...');
      try {
        const newEvent: DisplayEvent = {
          poolId: response.getPoolId(),
          eventType: response.getEventType(),
          payload: JSON.parse(response.getPayloadJson()),
          receivedAt: new Date().toLocaleTimeString(),
        };
        // Add new events to the top of the list
        setEvents(prevEvents => [newEvent, ...prevEvents]);
      } catch (e) {
        console.error("Failed to parse event payload:", e);
      }
    });

    stream.on('error', (err) => {
      console.error('gRPC Stream Error:', err);
      // Handle common connection errors for better UX
      if (err.code === 14) { // UNAVAILABLE
          setError(`Connection failed. Ensure the Heimdall stream server is running at http://localhost:50051 and that CORS is not blocking the request.`);
      } else {
          setError(`Stream Error (${err.code}): ${err.message}`);
      }
      setStatus('Error');
    });

    stream.on('end', () => {
      console.log('Stream ended by the server.');
      setStatus('Disconnected');
    });

    // CRITICAL: Cleanup function to close the stream when the component unmounts
    return () => {
      console.log('Canceling stream for', poolId);
      stream.cancel();
    };

  }, [poolId, status]); // Re-run effect if the poolId prop changes

  return (
    <div>
      <h3>Live Feed for Pool: <code>{poolId}</code></h3>
      <p><strong>Connection Status:</strong> {status}</p>
      {error && <p style={{ color: 'red' }}><strong>Error:</strong> {error}</p>}
      
      <div style={{ maxHeight: '600px', overflowY: 'auto', border: '1px solid #ccc', padding: '10px', background: '#f5f5f5', fontFamily: 'monospace' }}>
        {events.length === 0 && <p>Waiting for events...</p>}
        {events.map((event, index) => (
          <div key={index} style={{ borderBottom: '1px solid #ddd', paddingBottom: '10px', marginBottom: '10px' }}>
            <p><strong>Event:</strong> {event.eventType} @ {event.receivedAt}</p>
            <pre style={{ whiteSpace: 'pre-wrap', wordBreak: 'break-all' }}>
              <code>{JSON.stringify(event.payload, null, 2)}</code>
            </pre>
          </div>
        ))}
      </div>
    </div>
  );
};

export default PoolStreamViewer;
```

## 5. Usage Example

To use the component, simply import it into any page and provide the `pool_id` you want to monitor as a prop.

```typescript
import PoolStreamViewer from '../components/PoolStreamViewer';

export default function PoolDashboard() {
  // This could come from URL params, user input, etc.
  const targetPoolId = "ApvLFu3ecKSdabEcGK25vTMFoErPoKHK4XUVZDdcoeb4";

  return (
    <main>
      <h1>Pool Event Dashboard</h1>
      <PoolStreamViewer poolId={targetPoolId} />
    </main>
  );
}
```

This setup provides a robust, type-safe, and efficient way to stream real-time on-chain data directly to your frontend application. 