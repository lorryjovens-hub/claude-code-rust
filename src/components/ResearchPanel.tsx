import React from 'react';
import { X, Check, Loader2 } from 'lucide-react';
import { IconResearch } from './Icons';
import MarkdownRenderer from './MarkdownRenderer';

export interface ResearchSource {
    url: string;
    title: string;
    snippet?: string;
    favicon?: string;
}

export interface ResearchSubAgent {
    id: string;
    index?: number;
    sub_question: string;
    status: 'running' | 'done' | 'error';
    sources: ResearchSource[];
    findings: string;
    error?: string;
}

export interface ResearchData {
    plan?: { title?: string; sub_questions: string[] } | null;
    sub_agents: ResearchSubAgent[];
    sources: ResearchSource[];
    phase?: 'planning' | 'gathering' | 'writing' | null;
    phase_label?: string;
    report?: string | null;
    completed?: boolean;
    duration_ms?: number;
    error?: string;
}

interface Props {
    research: ResearchData;
    onClose: () => void;
}

function getFaviconUrl(url: string): string {
    try {
        const u = new URL(url);
        return `https://www.google.com/s2/favicons?domain=${u.hostname}&sz=32`;
    } catch {
        return '';
    }
}

function getDomain(url: string): string {
    try {
        return new URL(url).hostname.replace(/^www\./, '');
    } catch {
        return url;
    }
}

const PhaseHeader: React.FC<{ icon: React.ReactNode; label: string; subtle?: string }> = ({ icon, label, subtle }) => (
    <div className="flex items-center gap-2 mb-3">
        <div className="text-[#2E7CF6]">{icon}</div>
        <h3 className="text-[14px] font-semibold text-claude-text">{label}</h3>
        {subtle && <span className="text-[12px] text-claude-textSecondary">{subtle}</span>}
    </div>
);

const SourceChip: React.FC<{ source: ResearchSource }> = ({ source }) => {
    const favicon = getFaviconUrl(source.url);
    const domain = getDomain(source.url);
    return (
        <a
            href={source.url}
            target="_blank"
            rel="noopener noreferrer"
            className="flex items-center gap-1.5 px-2 py-1 rounded-md bg-claude-hover hover:bg-claude-btnHover transition-colors flex-shrink-0 max-w-[180px]"
            title={source.title}
        >
            {favicon ? (
                <img src={favicon} width={14} height={14} alt="" className="rounded-sm flex-shrink-0" />
            ) : (
                <div className="w-3.5 h-3.5 bg-claude-textSecondary rounded-sm flex-shrink-0" />
            )}
            <span className="text-[11px] text-claude-textSecondary truncate">{domain}</span>
        </a>
    );
};

const ResearchPanel: React.FC<Props> = ({ research, onClose }) => {
    const { plan, sub_agents = [], sources = [], phase, phase_label, report, completed, error } = research;

    return (
        <div className="h-full w-full flex flex-col bg-transparent">
            {/* Header */}
            <div className="flex items-center justify-between px-5 py-4 flex-shrink-0 border-b border-claude-border">
                <div className="flex items-center gap-2 min-w-0">
                    <IconResearch size={18} className="text-[#2E7CF6] flex-shrink-0" />
                    <h2 className="text-[15px] font-semibold text-claude-text truncate">
                        {plan?.title || 'Research'}
                    </h2>
                </div>
                <button
                    onClick={onClose}
                    className="p-1.5 text-claude-textSecondary hover:text-claude-text hover:bg-claude-hover rounded-lg transition-colors flex-shrink-0"
                    title="Close"
                >
                    <X size={18} />
                </button>
            </div>

            {/* Body */}
            <div className="flex-1 overflow-y-auto px-5 py-5 space-y-6">
                {/* Plan */}
                {plan && (
                    <section>
                        <PhaseHeader
                            icon={completed || sub_agents.length > 0 ? <Check size={16} /> : <Loader2 size={16} className="animate-spin" />}
                            label="Research plan created"
                            subtle={`${plan.sub_questions.length} sub-questions`}
                        />
                        <ol className="space-y-1.5 ml-6 list-decimal text-[12.5px] text-claude-textSecondary">
                            {plan.sub_questions.map((q, i) => (
                                <li key={i}>{q}</li>
                            ))}
                        </ol>
                    </section>
                )}

                {/* Sources gathered (global view) */}
                {sources.length > 0 && (
                    <section>
                        <PhaseHeader
                            icon={<Check size={16} />}
                            label={`Gathered ${sources.length} source${sources.length === 1 ? '' : 's'}`}
                        />
                        <div className="flex flex-wrap gap-1.5 ml-6">
                            {sources.map((s, i) => (
                                <SourceChip key={`${s.url}-${i}`} source={s} />
                            ))}
                        </div>
                    </section>
                )}

                {/* Sub-agents and their findings */}
                {sub_agents.map((sub) => (
                    <section key={sub.id} className="border-l-2 border-claude-border pl-4">
                        <div className="flex items-center gap-2 mb-2">
                            {sub.status === 'running' ? (
                                <Loader2 size={14} className="text-[#2E7CF6] animate-spin flex-shrink-0" />
                            ) : sub.status === 'error' ? (
                                <span className="text-red-500 text-[14px] flex-shrink-0">✗</span>
                            ) : (
                                <Check size={14} className="text-claude-textSecondary flex-shrink-0" />
                            )}
                            <h4 className="text-[13px] font-medium text-claude-text">{sub.sub_question}</h4>
                        </div>
                        {sub.sources.length > 0 && (
                            <div className="flex flex-wrap gap-1.5 mb-3">
                                {sub.sources.map((s, i) => (
                                    <SourceChip key={`${s.url}-${i}`} source={s} />
                                ))}
                            </div>
                        )}
                        {sub.findings && (
                            <div className="text-[12.5px] text-claude-textSecondary leading-relaxed prose-sm max-w-none">
                                <MarkdownRenderer content={sub.findings} />
                            </div>
                        )}
                        {sub.error && (
                            <div className="text-[12px] text-red-500 italic">Error: {sub.error}</div>
                        )}
                    </section>
                ))}

                {/* Phase indicator (when running) */}
                {phase && !completed && (
                    <section className="flex items-center gap-2 text-[12.5px] text-claude-textSecondary">
                        <Loader2 size={14} className="animate-spin text-[#2E7CF6]" />
                        <span>{phase_label || phase}</span>
                    </section>
                )}

                {/* Final report (also rendered in chat as message body, but mirrored here) */}
                {report && (
                    <section>
                        <PhaseHeader
                            icon={<Check size={16} />}
                            label="Final report"
                        />
                        <div className="ml-6 prose prose-sm dark:prose-invert max-w-none text-[13px]">
                            <MarkdownRenderer content={report} />
                        </div>
                    </section>
                )}

                {error && (
                    <section className="rounded-lg bg-red-50 dark:bg-red-900/20 border border-red-200 dark:border-red-800 p-3">
                        <p className="text-[12.5px] text-red-700 dark:text-red-300">Research error: {error}</p>
                    </section>
                )}

                {completed && !error && (
                    <section className="text-[11.5px] text-claude-textSecondary text-center pt-2 pb-4">
                        Research complete{research.duration_ms ? ` · ${(research.duration_ms / 1000).toFixed(1)}s` : ''}
                    </section>
                )}
            </div>
        </div>
    );
};

export default ResearchPanel;
