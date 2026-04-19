export function compileTemplateJson(templateJson: string): string;
export function bindCompiledJson(compiledJson: string, envJson: string, overlayJson?: string): string;
export function renderCompiledJson(compiledJson: string, bindingJson: string, mode: string): string;
