/*
 * Copyright © 2025-2026 rustmailer.com
 * Licensed under RustMailer License Agreement v1.0
 * Unauthorized use or distribution is prohibited.
 */

import React from 'react';
import AceEditor from 'react-ace';
import { cn } from '@/lib/utils';
import 'ace-builds/src-noconflict/mode-handlebars';
import 'ace-builds/src-noconflict/mode-json';
import 'ace-builds/src-noconflict/mode-python';
import 'ace-builds/src-noconflict/mode-markdown';
import 'ace-builds/src-noconflict/theme-kuroir';
import 'ace-builds/src-noconflict/theme-monokai';
import 'ace-builds/src-noconflict/ext-language_tools';

interface ReactAceEditorProps {
    value?: string;
    onChange?: (value: string) => void;
    placeholder?: string;
    className?: string;
    readOnly?: boolean;
    theme?: 'kuroir' | 'monokai';
    mode?: 'handlebars' | 'json' | 'markdown' | 'python';
}

const ReactAceEditor: React.FC<ReactAceEditorProps> = ({
    value,
    onChange,
    placeholder,
    className,
    readOnly = false,
    theme = 'github',
    mode = 'handlebars'
}) => {
    return (
        <div className={cn('w-full h-[100px]', className)}>
            <AceEditor
                mode={mode}
                theme={theme}
                readOnly={readOnly}
                value={value || ''}
                onChange={onChange}
                placeholder={placeholder}
                fontSize={14}
                showPrintMargin={false}
                showGutter={true}
                highlightActiveLine={true}
                width="100%"
                height="100%"
                setOptions={{
                    useWorker: false,
                    enableBasicAutocompletion: true,
                    enableMobileMenu: true,
                    enableLiveAutocompletion: false,
                    enableSnippets: false,
                    showLineNumbers: true,
                    tabSize: 2,
                }}
            />
        </div>
    );
};

export default ReactAceEditor;