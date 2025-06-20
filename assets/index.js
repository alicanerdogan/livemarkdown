let latestPosition = null;

function updateDocument(newContent) {
  const main = document.querySelector('main');
  if (!main) {
    return;
  }
  main.innerHTML = newContent;
}

function scrollToNewPosition(sourcepos) {
  const element = document.querySelector(`[data-sourcepos="${sourcepos}"]`);
  if (!element) {
    console.warn(`Element with data-sourcepos="${sourcepos}" not found`);
    return;
  }
  element.scrollIntoView({ behavior: 'smooth', block: 'center' });
}

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
    scrollToNewPosition(data.sourcepos);
    latestPosition = data.sourcepos;
  });
  eventSource.addEventListener('file_changed', (event) => {
    const data = JSON.parse(event.data);
    updateDocument(data.html);
    if (latestPosition) {
      scrollToNewPosition(latestPosition);
    }
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
