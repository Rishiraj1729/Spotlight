import { StrictMode } from 'react';
import { createRoot } from 'react-dom/client';
import { getCurrentWindow } from '@tauri-apps/api/window';
import './index.css';
import App from './App.tsx';
import SettingsApp from './SettingsApp.tsx';

const label = getCurrentWindow().label;
const Root = label === 'settings' ? SettingsApp : App;

createRoot(document.getElementById('root')!).render(
  <StrictMode>
    <Root />
  </StrictMode>,
);
