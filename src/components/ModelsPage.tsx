import React, { useState, useEffect } from 'react';
import { useNavigate } from 'react-router-dom';
import { ArrowLeft, Plus, Trash2, Check, X, Loader2, GripVertical } from 'lucide-react';
import { useI18n } from '../hooks/useI18n';

interface ChatModel {
  id: string;
  name: string;
  tier: string;
  enabled: number;
}

const ModelsPage = () => {
  const { t } = useI18n();
  const navigate = useNavigate();
  const [models, setModels] = useState<ChatModel[]>([]);
  const [loading, setLoading] = useState(true);
  const [showAddForm, setShowAddForm] = useState(false);
  const [saving, setSaving] = useState(false);
  const [newModel, setNewModel] = useState<ChatModel>({ id: '', name: '', tier: 'sonnet', enabled: 1 });

  useEffect(() => {
    loadModels();
  }, []);

  const loadModels = () => {
    setLoading(true);
    try {
      const saved = localStorage.getItem('chat_models');
      if (saved) {
        setModels(JSON.parse(saved));
      } else {
        setModels([]);
      }
    } catch (e) {
      setModels([]);
    } finally {
      setLoading(false);
    }
  };

  const saveModels = (newModels: ChatModel[]) => {
    localStorage.setItem('chat_models', JSON.stringify(newModels));
    setModels(newModels);
  };

  const handleAddModel = () => {
    if (!newModel.id.trim()) return;
    setSaving(true);
    const model: ChatModel = {
      id: newModel.id.trim(),
      name: newModel.name.trim() || newModel.id.trim(),
      tier: newModel.tier,
      enabled: 1,
    };
    const updated = [...models, model];
    saveModels(updated);
    setNewModel({ id: '', name: '', tier: 'sonnet', enabled: 1 });
    setShowAddForm(false);
    setSaving(false);
  };

  const handleDeleteModel = (id: string) => {
    if (!confirm(t('customize.confirmDeleteModel'))) return;
    const updated = models.filter(m => m.id !== id);
    saveModels(updated);
  };

  const handleToggleEnabled = (model: ChatModel) => {
    const updated = models.map(m =>
      m.id === model.id ? { ...m, enabled: m.enabled ? 0 : 1 } : m
    );
    saveModels(updated);
  };

  const handleMoveModel = (index: number, direction: 'up' | 'down') => {
    const newIndex = direction === 'up' ? index - 1 : index + 1;
    if (newIndex < 0 || newIndex >= models.length) return;
    const updated = [...models];
    [updated[index], updated[newIndex]] = [updated[newIndex], updated[index]];
    saveModels(updated);
  };

  const TIER_OPTIONS = [
    { value: 'opus', label: 'Opus 档' },
    { value: 'sonnet', label: 'Sonnet 档' },
    { value: 'haiku', label: 'Haiku 档' },
    { value: 'extra', label: '其他' },
  ];

  return (
    <div className="flex-1 h-full bg-claude-bg overflow-y-auto">
      <div className="max-w-[800px] mx-auto px-4 py-8 md:px-8 md:py-12">
        {/* Back button */}
        <button
          onClick={() => navigate('/')}
          className="flex items-center gap-1.5 text-claude-textSecondary hover:text-claude-text transition-colors mb-6"
        >
          <ArrowLeft size={16} />
          <span className="text-[14px]">{t('common.back')}</span>
        </button>

        {/* Header */}
        <div className="flex items-center justify-between mb-8">
          <div>
            <h1
              className="font-[Spectral] text-[32px] text-claude-text"
              style={{ fontWeight: 500, WebkitTextStroke: '0.5px currentColor' }}
            >
              {t('models.title')}
            </h1>
            <p className="text-[14px] text-claude-textSecondary mt-1">
              管理你的可用模型，拖拽调整顺序
            </p>
          </div>
          <button
            onClick={() => setShowAddForm(true)}
            className="flex items-center gap-2 px-3.5 py-1.5 bg-claude-text text-claude-bg hover:opacity-90 rounded-lg transition-opacity font-medium"
            style={{ fontSize: '14px' }}
          >
            <Plus size={16} />
            {t('customize.addModel')}
          </button>
        </div>

        {/* Add Model Form */}
        {showAddForm && (
          <div className="mb-8 p-5 bg-claude-input border border-claude-border rounded-xl">
            <h3 className="text-[15px] font-semibold text-claude-text mb-4">{t('customize.addNewModel')}</h3>
            <div className="grid grid-cols-2 gap-4 mb-4">
              <div>
                <label className="block text-[13px] text-claude-textSecondary mb-1.5">{t('customize.modelId')}</label>
                <input
                  type="text"
                  value={newModel.id}
                  onChange={(e) => setNewModel({ ...newModel, id: e.target.value })}
                  placeholder={t('customize.modelIdPlaceholder')}
                  className="w-full px-3 py-2 bg-transparent border border-claude-border rounded-lg text-[14px] text-claude-text focus:outline-none focus:border-blue-500"
                />
              </div>
              <div>
                <label className="block text-[13px] text-claude-textSecondary mb-1.5">{t('customize.displayName')}</label>
                <input
                  type="text"
                  value={newModel.name}
                  onChange={(e) => setNewModel({ ...newModel, name: e.target.value })}
                  placeholder={t('customize.displayNamePlaceholder')}
                  className="w-full px-3 py-2 bg-transparent border border-claude-border rounded-lg text-[14px] text-claude-text focus:outline-none focus:border-blue-500"
                />
              </div>
            </div>
            <div className="mb-4">
              <label className="block text-[13px] text-claude-textSecondary mb-1.5">{t('customize.tier')}</label>
              <div className="flex gap-2">
                {TIER_OPTIONS.map(opt => (
                  <button
                    key={opt.value}
                    onClick={() => setNewModel({ ...newModel, tier: opt.value })}
                    className={`px-3 py-1.5 rounded-lg text-[13px] font-medium transition-colors ${
                      newModel.tier === opt.value
                        ? 'bg-claude-text text-claude-bg'
                        : 'bg-claude-hover text-claude-textSecondary hover:text-claude-text'
                    }`}
                  >
                    {opt.label}
                  </button>
                ))}
              </div>
            </div>
            <div className="flex justify-end gap-3">
              <button
                onClick={() => { setShowAddForm(false); setNewModel({ id: '', name: '', tier: 'sonnet', enabled: 1 }); }}
                className="px-4 py-2 text-[14px] font-medium text-claude-text hover:bg-claude-hover rounded-lg transition-colors"
              >
                {t('common.cancel')}
              </button>
              <button
                onClick={handleAddModel}
                disabled={saving || !newModel.id.trim()}
                className="px-4 py-2 text-[14px] font-medium text-white bg-[#333333] hover:bg-[#1a1a1a] dark:bg-[#FFFFFF] dark:text-black dark:hover:bg-[#e5e5e5] rounded-lg transition-colors disabled:opacity-50 flex items-center gap-2"
              >
                {saving && <Loader2 size={14} className="animate-spin" />}
                {t('customize.addModel')}
              </button>
            </div>
          </div>
        )}

        {/* Models List */}
        {loading ? (
          <div className="flex items-center justify-center py-20">
            <Loader2 size={24} className="animate-spin text-claude-textSecondary" />
          </div>
        ) : models.length === 0 ? (
          <div className="flex flex-col items-center justify-center py-20 text-claude-textSecondary">
            <div className="w-16 h-16 rounded-2xl bg-claude-hover flex items-center justify-center mb-4">
              <Plus size={24} className="opacity-40" />
            </div>
            <p className="text-[15px] font-medium text-claude-text mb-1">{t('customize.noModelsConfigured')}</p>
            <p className="text-[13px]">{t('customize.addFirstModel')}</p>
          </div>
        ) : (
          <div className="space-y-3">
            {models.map((model, index) => (
              <div
                key={model.id}
                className="flex items-center justify-between p-4 bg-claude-input border border-claude-border rounded-xl hover:bg-claude-hover transition-colors group"
              >
                <div className="flex items-center gap-4">
                  <div className="flex flex-col gap-0.5 opacity-0 group-hover:opacity-100 transition-opacity">
                    <button
                      onClick={() => handleMoveModel(index, 'up')}
                      disabled={index === 0}
                      className="w-5 h-5 flex items-center justify-center text-claude-textSecondary hover:text-claude-text disabled:opacity-30 disabled:cursor-not-allowed"
                    >
                      <svg width="10" height="10" viewBox="0 0 10 10" fill="currentColor"><path d="M5 2L9 7H1L5 2Z" /></svg>
                    </button>
                    <button
                      onClick={() => handleMoveModel(index, 'down')}
                      disabled={index === models.length - 1}
                      className="w-5 h-5 flex items-center justify-center text-claude-textSecondary hover:text-claude-text disabled:opacity-30 disabled:cursor-not-allowed"
                    >
                      <svg width="10" height="10" viewBox="0 0 10 10" fill="currentColor"><path d="M5 8L1 3H9L5 8Z" /></svg>
                    </button>
                  </div>
                  <div className="w-10 h-10 rounded-lg bg-claude-btn-hover flex items-center justify-center flex-shrink-0">
                    <span className="text-[14px] font-semibold text-claude-text">
                      {model.tier?.charAt(0).toUpperCase() || 'M'}
                    </span>
                  </div>
                  <div>
                    <h3 className="text-[14px] font-medium text-claude-text">{model.name || model.id}</h3>
                    <p className="text-[12px] text-claude-textSecondary mt-0.5">{model.id}</p>
                  </div>
                </div>
                <div className="flex items-center gap-3">
                  <span className={`px-2.5 py-1 rounded-md text-[12px] font-medium ${
                    model.tier === 'opus' ? 'bg-purple-500/10 text-purple-400' :
                    model.tier === 'sonnet' ? 'bg-blue-500/10 text-blue-400' :
                    model.tier === 'haiku' ? 'bg-green-500/10 text-green-400' :
                    'bg-claude-hover text-claude-textSecondary'
                  }`}>
                    {model.tier || 'extra'}
                  </span>
                  <button
                    onClick={() => handleToggleEnabled(model)}
                    className={`w-8 h-8 rounded-lg flex items-center justify-center transition-colors ${
                      model.enabled
                        ? 'text-green-500 hover:bg-green-500/10'
                        : 'text-claude-textSecondary hover:bg-claude-hover'
                    }`}
                    title={model.enabled ? t('customize.disable') : t('customize.enable')}
                  >
                    {model.enabled ? <Check size={16} /> : <X size={16} />}
                  </button>
                  <button
                    onClick={() => handleDeleteModel(model.id)}
                    className="w-8 h-8 rounded-lg flex items-center justify-center text-claude-textSecondary hover:text-[#B9382C] hover:bg-[#B9382C]/10 transition-colors"
                    title={t('common.delete')}
                  >
                    <Trash2 size={16} />
                  </button>
                </div>
              </div>
            ))}
          </div>
        )}
      </div>
    </div>
  );
};

export default ModelsPage;
