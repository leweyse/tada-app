import { StrictMode } from 'react';
import { createRoot } from 'react-dom/client';

import reactLogo from './assets/react.svg';

import './index.css';

function App() {
  return (
    <div style={{ display: 'grid', placeItems: 'center', gap: '1rem' }}>
      <a href='https://react.dev' target='_blank' rel='noreferrer'>
        <img
          src={reactLogo}
          alt='Solid logo'
          style={{
            height: '6rem',
            padding: '1rem',
            filter: 'drop-shadow(0 0 2em #61dafbaa)',
          }}
        />
      </a>
      <h1
        style={{
          marginBlock: 0,
          marginInline: 0,
          fontWeight: 'bold',
          fontSize: '3.2em',
          lineHeight: 1.1,
        }}
      >
        Vite + React
      </h1>
    </div>
  );
}

const root = document.getElementById('app');

createRoot(root!).render(
  <StrictMode>
    <App />
  </StrictMode>,
);
