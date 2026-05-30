import { X } from 'lucide-react';

interface CreateProjectDialogProps {
  newProjectName: string;
  newProjectDescription: string;
  onNameChange: (v: string) => void;
  onDescriptionChange: (v: string) => void;
  onCreate: () => void;
  onClose: () => void;
}

export default function CreateProjectDialog({
  newProjectName,
  newProjectDescription,
  onNameChange,
  onDescriptionChange,
  onCreate,
  onClose,
}: CreateProjectDialogProps) {
  return (
    <div
      className="fixed inset-0 z-[200] flex items-center justify-center bg-black/40"
      onClick={onClose}
    >
      <div
        className="bg-claude-bg border border-claude-border rounded-2xl shadow-xl w-[560px] max-w-[92vw] overflow-hidden"
        onClick={(e) => e.stopPropagation()}
      >
        <div className="flex items-start justify-between px-7 pt-6 pb-4">
          <h2
            className="font-[Spectral] text-[22px] text-claude-text"
            style={{ fontWeight: 600 }}
          >
            Create a project
          </h2>
          <button
            onClick={onClose}
            className="p-1 text-claude-textSecondary hover:text-claude-text hover:bg-claude-hover rounded-lg transition-colors"
          >
            <X size={18} />
          </button>
        </div>
        <div className="px-7 pb-4 space-y-5">
          <div>
            <label className="block text-[15px] font-medium text-claude-textSecondary mb-2">
              What are you working on?
            </label>
            <input
              type="text"
              placeholder="Name your project"
              value={newProjectName}
              onChange={(e) => onNameChange(e.target.value)}
              onKeyDown={(e) => {
                if (e.key === 'Enter' && newProjectName.trim()) onCreate();
              }}
              className="w-full px-4 py-3 bg-white dark:bg-claude-input border border-gray-200 dark:border-claude-border rounded-xl text-claude-text placeholder-gray-400 dark:placeholder-gray-500 focus:outline-none focus:border-[#387ee0] focus:ring-0 transition-all text-[15px]"
              autoFocus
            />
          </div>
          <div>
            <label className="block text-[15px] font-medium text-claude-textSecondary mb-2">
              What are you trying to achieve?
            </label>
            <textarea
              placeholder="Describe your project, goals, subject, etc..."
              rows={3}
              value={newProjectDescription}
              onChange={(e) => onDescriptionChange(e.target.value)}
              className="w-full px-4 py-3 bg-white dark:bg-claude-input border border-gray-200 dark:border-claude-border rounded-xl text-claude-text placeholder-gray-400 dark:placeholder-gray-500 focus:outline-none focus:border-[#387ee0] focus:ring-0 transition-all text-[15px] resize-none"
            />
          </div>
        </div>
        <div className="flex items-center justify-end gap-3 px-7 pb-6 pt-2">
          <button
            onClick={onClose}
            className="px-5 py-2.5 text-[15px] font-medium text-claude-text bg-white dark:bg-claude-bg border border-gray-300 dark:border-claude-border hover:bg-gray-50 dark:hover:bg-claude-hover rounded-xl transition-colors"
          >
            Cancel
          </button>
          <button
            onClick={onCreate}
            disabled={!newProjectName.trim()}
            className="px-5 py-2.5 text-[15px] font-medium text-claude-bg bg-black dark:bg-white dark:text-black hover:opacity-90 rounded-xl transition-opacity disabled:opacity-40"
          >
            Create project
          </button>
        </div>
      </div>
    </div>
  );
}
