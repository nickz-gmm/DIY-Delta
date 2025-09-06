import { invoke } from '@tauri-apps/api/core'

export const startF1 = (port:number=20777, format:number=2025) => invoke('start_f1', { port, format })
export const startGT7 = (consoleIp:string, variant:string='A', bindPort:number=33740) => invoke('start_gt7', { consoleIp, variant, bindPort })
export const startLMU = () => invoke('start_lmu')

export const stopAll = () => invoke('stop_all')

export const listLaps = () => invoke('list_laps') as Promise<any[]>
export const analyzeLaps = (ids: string[]) => invoke('analyze_laps', { lapIds: ids })
export const buildTrackMap = (id: string) => invoke('build_track_map', { lapId: id })

export const importFile = (path: string) => invoke('import_file', { path })
export const exportFile = (kind: 'csv'|'ndjson'|'motec_csv', path:string) => invoke('export_file', { kind, path })

export const carsAndTracks = (game: string) => invoke('cars_and_tracks', { game })

export const saveWorkspace = (name: string, payload: any) => invoke('save_workspace', { name, payload })
export const loadWorkspace = (name: string) => invoke('load_workspace', { name })
export const listWorkspaces = () => invoke('list_workspaces') as Promise<string[]>
