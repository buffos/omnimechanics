import { Pipe, PipeTransform, inject } from '@angular/core';
import { DomSanitizer, SafeHtml } from '@angular/platform-browser';
import katex from 'katex';

@Pipe({
  name: 'katex',
  standalone: true,
})
export class KatexPipe implements PipeTransform {
  private readonly sanitizer = inject(DomSanitizer);

  transform(tex: string | null | undefined, displayMode = false): SafeHtml {
    if (!tex) return '';
    try {
      // Auto-escape bare '&' inside \text{...} so KaTeX does not choke on text like \text{Cross & Kani}
      const sanitizedTex = tex.replace(/(\\text\{[^}]*?)(?<!\\)&/g, '$1\\&');
      const html = katex.renderToString(sanitizedTex, {
        throwOnError: false,
        displayMode,
      });
      return this.sanitizer.bypassSecurityTrustHtml(html);
    } catch {
      return tex;
    }
  }
}
