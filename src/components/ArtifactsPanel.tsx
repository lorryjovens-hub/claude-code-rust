import React from 'react';
import { X } from 'lucide-react';
import DocumentCard, { DocumentInfo } from './DocumentCard';

interface ArtifactsPanelProps {
    documents: DocumentInfo[];
    onClose: () => void;
    onOpenDocument: (doc: DocumentInfo) => void;
}

const ArtifactsPanel: React.FC<ArtifactsPanelProps> = ({ documents, onClose, onOpenDocument }) => {
    return (
        <div className="h-full w-full flex flex-col bg-transparent">
            {/* Header */}
            <div className="flex items-center justify-between px-5 py-4 flex-shrink-0">
                <h2 className="text-[16px] font-semibold text-claude-text">Artifacts</h2>
                <button
                    onClick={onClose}
                    className="p-1.5 text-claude-textSecondary hover:text-claude-text hover:bg-claude-btn-hover rounded-lg transition-colors"
                    title="Close"
                >
                    <X size={18} />
                </button>
            </div>

            {/* Content List */}
            <div className="flex-1 overflow-y-auto px-4 pb-4">
                {documents.length === 0 ? (
                    <div className="text-center text-claude-textSecondary mt-10 text-[14px]">
                        No artifacts generated in this chat yet.
                    </div>
                ) : (
                    <div className="space-y-3">
                        {documents.map((doc, idx) => (
                            <DocumentCard
                                key={doc.id || idx}
                                document={doc}
                                onOpen={onOpenDocument}
                            />
                        ))}
                    </div>
                )}
            </div>
        </div>
    );
};

export default ArtifactsPanel;
