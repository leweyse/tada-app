/* @refresh reload */
import { render } from 'solid-js/web';

import solidLogo from './assets/solid.svg';

import './index.css';

function App() {
  return (
    <div style={{ display: 'grid', 'place-items': 'center', gap: '1rem' }}>
      <a href='https://solidjs.com' target='_blank' rel='noreferrer'>
        <img
          src={solidLogo}
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
          'margin-block': 0,
          'margin-inline': 0,
          'font-weight': 'bold',
          'font-size': '3.2em',
          'line-height': 1.1,
        }}
      >
        Vite + Solid
      </h1>
    </div>
  );
}

const root = document.getElementById('app');

render(() => <App />, root!);
