/**
 * Carnelia Docs - Collaborative Document Editor
 * 
 * A real-time collaborative document editor powered by CRDTs.
 */

import { useState, useEffect } from 'react';
import { Editor } from './components/Editor';
import { initCarnelia, generateDocId } from './lib/carnelia-client';

function App() {
  const [isLoading, setIsLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);
  const [docId, setDocId] = useState<string>('');
  const [userName, setUserName] = useState<string>('');
  const [isEditing, setIsEditing] = useState(false);

  // Initialize WASM on mount
  useEffect(() => {
    const init = async () => {
      try {
        await initCarnelia();
        setIsLoading(false);
        
        // Check URL for doc ID
        const params = new URLSearchParams(window.location.search);
        const urlDocId = params.get('doc');
        if (urlDocId) {
          setDocId(urlDocId);
          setIsEditing(true);
        }
      } catch (err) {
        setError('Failed to load editor. Please refresh the page.');
        console.error('WASM init error:', err);
      }
    };
    
    init();
  }, []);

  // Create new document
  const handleCreateDoc = () => {
    const newDocId = generateDocId();
    setDocId(newDocId);
    
    // Update URL
    const url = new URL(window.location.href);
    url.searchParams.set('doc', newDocId);
    window.history.pushState({}, '', url.toString());
    
    setIsEditing(true);
  };

  // Join existing document
  const handleJoinDoc = (e: React.FormEvent) => {
    e.preventDefault();
    if (docId.trim()) {
      // Update URL
      const url = new URL(window.location.href);
      url.searchParams.set('doc', docId.trim());
      window.history.pushState({}, '', url.toString());
      
      setIsEditing(true);
    }
  };

  // Loading state
  if (isLoading) {
    return (
      <div className="min-h-screen bg-linear-to-br from-blue-50 to-indigo-100 flex items-center justify-center">
        <div className="text-center">
          <div className="w-16 h-16 border-4 border-blue-500 border-t-transparent rounded-full animate-spin mx-auto mb-4" />
          <h2 className="text-xl font-semibold text-gray-700">Loading Carnelia Editor...</h2>
          <p className="text-gray-500 mt-2">Initializing CRDT engine</p>
        </div>
      </div>
    );
  }

  // Error state
  if (error) {
    return (
      <div className="min-h-screen bg-red-50 flex items-center justify-center">
        <div className="text-center max-w-md p-8">
          <div className="text-5xl mb-4">⚠️</div>
          <h2 className="text-xl font-semibold text-red-700 mb-2">Error</h2>
          <p className="text-red-600">{error}</p>
          <button
            onClick={() => window.location.reload()}
            className="mt-4 px-4 py-2 bg-red-600 text-white rounded-lg hover:bg-red-700 transition"
          >
            Reload Page
          </button>
        </div>
      </div>
    );
  }

  // Editor view
  if (isEditing && docId) {
    return (
      <div className="min-h-screen bg-white flex flex-col">
        <header className="bg-[#1E1E1E] text-white px-6 py-3 flex items-center justify-between shadow-lg">
          <button
            onClick={() => {
              setIsEditing(false);
              setDocId('');
              window.history.pushState({}, '', window.location.pathname);
            }}
            className="px-2 bg-white/20 hover:bg-white/30 rounded-lg text-sm transition"
          >
            ←
          </button>
          <div className="flex items-center gap-3">
            <h1 className="text-xl font-bold">Carnelia Docs</h1>
          </div>
          <div className="flex items-center gap-4">
            <button
              onClick={() => {
                navigator.clipboard.writeText(window.location.href);
              }}
              className="px-3 py-1.5 bg-white/20 hover:bg-white/30 rounded-lg text-sm transition"
            >
              Copy Link
            </button>
          </div>
        </header>
        
        <main className="flex-1 overflow-hidden">
          <Editor docId={docId} userName={userName || 'Anonymous'} />
        </main>
      </div>
    );
  }

  // Landing page
  return (
    <div className="min-h-screen bg-linear-to-br from-blue-50 to-indigo-100">
      <div className="container mx-auto px-4 py-16">
        {/* Hero */}
        <div className="text-center mb-16">
          <div className="text-6xl mb-4">📝</div>
          <h1 className="text-4xl font-bold text-gray-800 mb-4">
            Carnelia Docs
          </h1>
          <p className="text-xl text-gray-600 max-w-2xl mx-auto">
            Real-time collaborative document editing powered by CRDTs.
            <br />
            <span className="text-blue-600 font-medium">Offline-first. Conflict-free. Always in sync.</span>
          </p>
        </div>

        {/* Action cards */}
        <div className="max-w-3xl mx-auto grid md:grid-cols-2 gap-8">
          {/* Create new */}
          <div className="bg-white rounded-2xl shadow-xl p-8">
            <h2 className="text-2xl font-bold text-gray-800 mb-4">
              Create New Document
            </h2>
            <p className="text-gray-600 mb-6">
              Start a fresh collaborative document and invite others to edit with you.
            </p>
            
            <div className="space-y-4">
              <input
                type="text"
                placeholder="Your name (optional)"
                value={userName}
                onChange={(e) => setUserName(e.target.value)}
                className="w-full px-4 py-3 border border-gray-300 rounded-lg focus:ring-2 focus:ring-blue-500 focus:border-blue-500 outline-none"
              />
              <button
                onClick={handleCreateDoc}
                className="w-full py-3 bg-blue-600 text-white font-semibold rounded-lg hover:bg-blue-700 transition shadow-lg shadow-blue-200"
              >
                Create Document →
              </button>
            </div>
          </div>

          {/* Join existing */}
          <div className="bg-white rounded-2xl shadow-xl p-8">
            <h2 className="text-2xl font-bold text-gray-800 mb-4">
              Join Existing Document
            </h2>
            <p className="text-gray-600 mb-6">
              Enter a document ID to join an existing collaborative session.
            </p>
            
            <form onSubmit={handleJoinDoc} className="space-y-4">
              <input
                type="text"
                placeholder="Your name (optional)"
                value={userName}
                onChange={(e) => setUserName(e.target.value)}
                className="w-full px-4 py-3 border border-gray-300 rounded-lg focus:ring-2 focus:ring-indigo-500 focus:border-indigo-500 outline-none"
              />
              <input
                type="text"
                placeholder="Document ID"
                value={docId}
                onChange={(e) => setDocId(e.target.value)}
                className="w-full px-4 py-3 border border-gray-300 rounded-lg focus:ring-2 focus:ring-indigo-500 focus:border-indigo-500 outline-none"
              />
              <button
                type="submit"
                disabled={!docId.trim()}
                className="w-full py-3 bg-indigo-600 text-white font-semibold rounded-lg hover:bg-indigo-700 transition shadow-lg shadow-indigo-200 disabled:opacity-50 disabled:cursor-not-allowed"
              >
                Join Document →
              </button>
            </form>
          </div>
        </div>

        {/* Features */}
        <div className="mt-20 text-center">
          <h2 className="text-2xl font-bold text-gray-800 mb-8">
            Why Carnelia?
          </h2>
          <div className="grid md:grid-cols-3 gap-8 max-w-4xl mx-auto">
            <div className="p-6">
              <div className="text-4xl mb-4">🔄</div>
              <h3 className="font-semibold text-lg text-gray-800 mb-2">Conflict-Free</h3>
              <p className="text-gray-600">
                CRDT technology ensures all edits merge automatically without conflicts.
              </p>
            </div>
            <div className="p-6">
              <div className="text-4xl mb-4">📴</div>
              <h3 className="font-semibold text-lg text-gray-800 mb-2">Offline-First</h3>
              <p className="text-gray-600">
                Keep editing even without internet. Changes sync when you reconnect.
              </p>
            </div>
            <div className="p-6">
              <div className="text-4xl mb-4">⚡</div>
              <h3 className="font-semibold text-lg text-gray-800 mb-2">Instant Sync</h3>
              <p className="text-gray-600">
                See collaborators' changes in real-time with zero delay.
              </p>
            </div>
          </div>
        </div>

        {/* Footer */}
        <footer className="mt-20 text-center text-gray-500 text-sm">
          <p>
            Built with <span className="text-red-500">♥</span> using{' '}
            <a 
              href="https://github.com/Agate-DB/Carnelia" 
              className="text-blue-600 hover:underline"
              target="_blank"
              rel="noopener noreferrer"
            >
              Carnelia CRDT
            </a>
          </p>
        </footer>
      </div>
    </div>
  );
}

export default App;
