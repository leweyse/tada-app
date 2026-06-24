import { createFileRoute } from '@tanstack/react-router';

import reactLogo from '@/assets/react.svg?url';

export const Route = createFileRoute('/')({
  component: HomePage,
});

function HomePage() {
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
        Tanstack Start
      </h1>
    </div>
  );
}
