export function Icon({ name, size = 20 }: { name: string; size?: number }) {
  const paths: Record<string, React.ReactNode> = {
    day: <><rect x="3" y="4" width="18" height="17" rx="2"/><path d="M3 10h18M8 2v4m8-4v4m-9 8h4m-4 3h7"/></>,
    history: <><path d="M3 11a9 9 0 1 1 2 7M3 4v7h7"/><path d="M12 7v5l3 2"/></>,
    settings: <><path d="M4 7h16M4 17h16"/><circle cx="9" cy="7" r="3"/><circle cx="15" cy="17" r="3"/></>,
    pause: <><path d="M8 5v14M16 5v14"/></>,
    play: <path d="m8 4 12 8-12 8Z"/>,
    download: <><path d="M12 3v12m-5-5 5 5 5-5M4 16v5h16v-5"/></>,
    arrow: <path d="m9 5 7 7-7 7"/>,
    monitor: <><rect x="3" y="3" width="18" height="13" rx="2"/><path d="M8 21h8m-4-5v5"/></>,
  };
  return <svg aria-hidden="true" width={size} height={size} viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="1.7" strokeLinecap="round" strokeLinejoin="round">{paths[name] ?? paths.day}</svg>;
}
