(function() {
  if (!window.location.pathname.startsWith('/document/')) {
    return;
  }

  const documentId = window.location.pathname.split('/').at(-1);
  if (!documentId) {
    console.error('Document ID not found in URL path');
    return;
  }

  const eventSource = new EventSource(`/document/${documentId}/updates`);
  eventSource.addEventListener('position', (event) => {
    const data = JSON.parse(event.data);
    console.log('Position update:', data);
  });
  eventSource.addEventListener('file_changed', (event) => {
    console.log('File changed:', event.data);
  });
  eventSource.addEventListener('error', (event) => {
    console.error('SSE connection error:', event);
  });
  eventSource.addEventListener('open', (event) => {
    console.log('SSE connection opened');
  });
  window.addEventListener('beforeunload', () => {
    eventSource.close();
  });
})();
