export interface Settings { paused: boolean; configured: boolean; idle_minutes: number; vdi: { executable: string; window_class: string } | null }
export interface Activity { source: string; hostname: string | null; app_name: string | null; reason: string }
export interface Status { settings: Settings; activity: Activity; error: string | null; warning: string | null; extension_connected: boolean; vdi_in_focus: boolean; last_saved_utc: number | null; data_path: string }
export interface Segment extends Activity { id: number; run_id: string; start_utc: number; end_utc: number; duration_ms: number; local_date: string; offset_seconds: number; is_open: boolean }
export interface Category { id: string; name: string; distraction: boolean }
export interface DomainTotal { hostname: string; category: string; duration_ms: number; accesses: number }
export interface AppTotal { label: string; category: string; duration_ms: number }
export interface Bucket { label: string; duration_ms: number }
export interface DayReport { schema_version: number; date: string; sessions: Segment[]; categories: Category[]; domains: DomainTotal[]; app_totals: AppTotal[]; category_totals: Bucket[]; hourly: Bucket[]; vdi_ms: number; idle_ms: number; unknown_ms: number; unclassified_ms: number; unidentified_ms: number; coverage_ms: number; distraction_ms: number | null; largest_distraction: string | null; longest_focus_ms: number; context_switches: number }
