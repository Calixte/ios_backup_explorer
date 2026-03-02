/// <reference lib="webworker" />

export type { };
declare const self: ServiceWorkerGlobalScope;

const streams = new Map<string, ReadableStream>();

self.oninstall = () => self.skipWaiting();
self.onactivate = () => self.clients.claim();

self.onmessage = (event: ExtendableMessageEvent) => {
    if (event.data && event.data.type === 'PORT_TRANSFER') {
        const { id, stream } = event.data;
        streams.set(id, stream);
    } else if (event.data && event.data.type === 'HEARTBEAT') {
        console.debug('Heartbeat received');
    }
};

self.onfetch = (event: FetchEvent) => {
    const url = new URL(event.request.url);
    if (url.pathname.endsWith('/download-stream')) {
        const id = url.searchParams.get('id');
        if (id) {
            const stream = streams.get(id);
            if (stream) {
                streams.delete(id);
                const { readable, writable } = new TransformStream();
                const promise = stream.pipeTo(writable);
                event.respondWith(new Response(readable, {
                    headers: {
                        'Content-Type': 'application/zip',
                        'Content-Disposition': `attachment; filename="backup.zip"`,
                    }
                }));
            }
        }
    }
};
