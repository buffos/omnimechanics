import { Injectable, computed, signal } from '@angular/core';

export type Language = 'en' | 'el';

const TRANSLATIONS: Record<Language, Record<string, string>> = {
  en: {
    'nav.continuum': '🔬 Continuum Workbench',
    'nav.teaching': '🎓 Teaching Studio',
    'studio.title': 'Educational Studio',
    'studio.subtitle': 'Structural & Continuum Mechanics',
    'studio.selectPreset': 'Select Structure / Preset:',
    'studio.length': 'Span Length L (m):',
    'studio.height': 'Truss Height H (m):',
    'studio.secondSpan': 'Second Span / Overhang (m):',
    'studio.distLoad': 'Distributed Load q (kN/m):',
    'studio.pointLoad': 'Point Load P (kN):',
    'studio.pointLoadPos': 'Point Load Position x_P (m):',
    'studio.recalculate': '⚡ Re-calculate & Steps',
    'studio.calculating': '⏳ Calculating...',
    'walker.first': '⏮ First',
    'walker.prev': '◀ Previous',
    'walker.next': 'Next ▶',
    'walker.last': 'Last ⏭',
    'walker.step': 'Step',
    'walker.of': 'of',
    'walker.play': '▶ Auto-Play',
    'walker.pause': '⏸ Pause',
    'walker.diagram': 'Diagram:',
    'walker.diagramM': 'M (Moment)',
    'walker.diagramQ': 'Q (Shear)',
    'walker.diagramN': 'N (Axial)',
    'walker.symbolic': 'Symbolic Equation:',
    'walker.substitution': 'Numerical Substitution:',
    'walker.result': 'Step Result:',
    'phase.DegreeOfIndeterminacy': 'Degree of Indeterminacy',
    'phase.SupportReactions': 'Support Reactions',
    'phase.MethodOfJoints': 'Method of Joints',
    'phase.MethodOfSections': 'Method of Sections (Ritter)',
    'phase.InternalForcesPiecewise': 'Internal Forces Q(x) & M(x)',
    'phase.CriticalExtrema': 'Critical Extrema (M_max)',
    'phase.CrossIteration': 'Cross Method',
    'phase.Default': 'Educational Step',
    'cross.title': 'Moment Distribution Cycles Table (Cross Method)',
    'cross.phase': 'Phase / Cycle',
    'cross.fem': 'Fixed-End Moments (FEM)',
    'cross.final': 'Final Support Moments',
  },
  el: {
    'nav.continuum': '🔬 Υπολογιστικό Συνεχούς',
    'nav.teaching': '🎓 Εκπαιδευτικό Studio',
    'studio.title': 'Εκπαιδευτικό Studio',
    'studio.subtitle': 'Στατική & Δυναμική Μηχανική',
    'studio.selectPreset': 'Επιλογή Φορέα / Προτύπου:',
    'studio.length': 'Μήκος Ανοίγματος L (m):',
    'studio.height': 'Ύψος Δικτυώματος H (m):',
    'studio.secondSpan': 'Δεύτερο Άνοιγμα / Πρόβολος (m):',
    'studio.distLoad': 'Κατανεμημένο Φορτίο q (kN/m):',
    'studio.pointLoad': 'Συγκεντρωμένο Φορτίο P (kN):',
    'studio.pointLoadPos': 'Θέση Συγκεντρωμένου Φορτίου x_P (m):',
    'studio.recalculate': '⚡ Επαναϋπολογισμός & Βήματα',
    'studio.calculating': '⏳ Επίλυση...',
    'walker.first': '⏮ Πρώτο',
    'walker.prev': '◀ Προηγούμενο',
    'walker.next': 'Επόμενο ▶',
    'walker.last': 'Τελευταίο ⏭',
    'walker.step': 'Βήμα',
    'walker.of': 'από',
    'walker.play': '▶ Auto-Play',
    'walker.pause': '⏸ Παύση',
    'walker.diagram': 'Διάγραμμα:',
    'walker.diagramM': 'M (Ροπή)',
    'walker.diagramQ': 'Q (Τέμνουσα)',
    'walker.diagramN': 'N (Αξονική)',
    'walker.symbolic': 'Συμβολική Εξίσωση:',
    'walker.substitution': 'Αριθμητική Αντικατάσταση:',
    'walker.result': 'Αποτέλεσμα Βήματος:',
    'phase.DegreeOfIndeterminacy': 'Στατική Οριστότητα',
    'phase.SupportReactions': 'Αντιδράσεις Στήριξης',
    'phase.MethodOfJoints': 'Μέθοδος Κόμβων',
    'phase.MethodOfSections': 'Μέθοδος Ritter (Τομών)',
    'phase.InternalForcesPiecewise': 'Διαγράμματα Q(x) & M(x)',
    'phase.CriticalExtrema': 'Ακρότατες Τιμές (M_max)',
    'phase.CrossIteration': 'Μέθοδος Cross',
    'phase.Default': 'Εκπαιδευτικό Βήμα',
    'cross.title': 'Πίνακας Διαδοχικών Κύκλων Κατανομής Ροπών (Μέθοδος Cross)',
    'cross.phase': 'Φάση / Κύκλος',
    'cross.fem': 'Ροπές Τέλους Πάκτωσης (FEM)',
    'cross.final': 'Τελικές Ροπές Στηρίξεων',
  },
};

@Injectable({
  providedIn: 'root',
})
export class I18nService {
  readonly currentLang = signal<Language>('el'); // default to Greek as requested, with instant toggle to English

  setLanguage(lang: Language): void {
    this.currentLang.set(lang);
  }

  toggleLanguage(): void {
    this.currentLang.update((l) => (l === 'el' ? 'en' : 'el'));
  }

  t(key: string): string {
    const lang = this.currentLang();
    return TRANSLATIONS[lang]?.[key] ?? TRANSLATIONS.en[key] ?? key;
  }
}
