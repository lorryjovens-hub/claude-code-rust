import { useEffect } from 'react';
import { draftsStore } from '../message-list/MessageList';

export function useDraftPersistence(
  activeId: string | null | undefined,
  inputBarBaseHeight: number,
  inputTextRef: React.MutableRefObject<string>,
  pendingFilesRef: React.MutableRefObject<any[]>,
  textareaHeightRef: React.MutableRefObject<number>,
  inputRef: React.MutableRefObject<HTMLTextAreaElement | null>,
  setInputText: (v: string) => void,
  setPendingFiles: (v: any[]) => void,
) {
  const draftKey = activeId || '__new__';
  useEffect(() => {
    const saved = draftsStore.get(draftKey);
    if (saved) {
      setInputText(saved.text);
      setPendingFiles(saved.files);
      textareaHeightRef.current = saved.height;
      if (inputRef.current) {
        inputRef.current.style.height = `${saved.height}px`;
        inputRef.current.style.overflowY = saved.height >= 316 ? 'auto' : 'hidden';
      }
      draftsStore.delete(draftKey);
    } else {
      setInputText('');
      setPendingFiles([]);
      textareaHeightRef.current = inputBarBaseHeight;
    }
    return () => {
      const text = inputTextRef.current;
      const files = pendingFilesRef.current;
      const height = textareaHeightRef.current;
      if (text.trim() || files.length > 0) {
        draftsStore.set(draftKey, { text, files, height });
      } else {
        draftsStore.delete(draftKey);
      }
    };
  }, [draftKey, inputBarBaseHeight]);
}
