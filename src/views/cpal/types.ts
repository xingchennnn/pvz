export type AudioStatsSnapshot = {
  input_overflows: number;
  output_underflows: number;
  inference_frames: number;
  inference_errors: number;
  average_inference_ms: number;
  last_inference_ms: number;
};

export type AudioStatus = {
  running: boolean;
  listening: boolean;
  sample_rate: number;
  model_sample_rate: number;
  input_device_name: string | null;
  virtual_output_device_name: string | null;
  monitor_output_device_name: string | null;
  input_channels: number | null;
  virtual_output_channels: number | null;
  monitor_output_channels: number | null;
  input_sample_format: string | null;
  virtual_output_sample_format: string | null;
  monitor_output_sample_format: string | null;
  stats: AudioStatsSnapshot;
};

export type AudioDeviceInfo = {
  id: string;
  name: string;
  detail: string | null;
  compatible: boolean;
  supported_sample_formats: string[];
  virtual_mic_candidate: boolean;
};

export type AudioDeviceCatalog = {
  model_sample_rate: number;
  default_input_id: string | null;
  default_output_id: string | null;
  preferred_virtual_output_id: string | null;
  preferred_monitor_output_id: string | null;
  inputs: AudioDeviceInfo[];
  outputs: AudioDeviceInfo[];
};

export type AudioRouteSelection = {
  inputDeviceId: string;
  virtualOutputDeviceId: string;
  monitorOutputDeviceId: string;
};

export type VBCableStatus = {
  installed: boolean;
  package_dir: string | null;
  setup_path: string | null;
  control_panel_path: string | null;
  input_device_id: string | null;
  input_device_name: string | null;
  output_device_id: string | null;
  output_device_name: string | null;
  official_download_url: string;
  integration_notice: string;
};

export type SavedAudioRouteSelection = Partial<
  AudioRouteSelection & {
    inputDeviceName: string;
    virtualOutputDeviceName: string;
    monitorOutputDeviceName: string;
  }
>;

export type AudioSelectOption = {
  id: string;
  label: string;
  recommended?: boolean;
};

export type AudioDevicePanelItem = {
  id: string;
  label: string;
  badgeLabel: string;
  badgeVariant: "success" | "muted" | "primary";
  lines: string[];
};