import React, { useState } from 'react';
import styled from '@emotion/styled';
import Button from '../common/Button';

const Wrapper = styled.div`
  display: flex;
  flex-direction: column;
`;

const Waveform = styled.div<{ active: boolean }>`
  height: 20px;
  margin-top: 0.5rem;
  background: var(--color-primary);
  opacity: ${({ active }) => (active ? 1 : 0.2)};
  transition: opacity 0.3s;
`;

const HistoryList = styled.ul`
  list-style: none;
  padding: 0;
  margin: 0.5rem 0 0 0;
`;

const VoiceControls: React.FC = () => {
  const [mode, setMode] = useState<'push' | 'continuous'>('push');
  const [listening, setListening] = useState(false);
  const [transcript, setTranscript] = useState('');
  const [confidence, setConfidence] = useState(0);
  const [history, setHistory] = useState<{ text: string; confidence: number }[]>(
    []
  );

  const toggleMode = () => setMode((m) => (m === 'push' ? 'continuous' : 'push'));

  const startListening = () => {
    setListening(true);
    // placeholder transcription
    setTimeout(() => {
      const text = 'Example command';
      const conf = 0.95;
      setTranscript(text);
      setConfidence(conf);
    }, 1000);
  };

  const stopListening = () => {
    setListening(false);
    if (transcript) {
      setHistory([...history, { text: transcript, confidence }]);
      setTranscript('');
      setConfidence(0);
    }
  };

  return (
    <Wrapper>
      <div style={{ display: 'flex', alignItems: 'center' }}>
        <span>Mode: {mode === 'push' ? 'Push-to-talk' : 'Continuous'}</span>
        <Button onClick={toggleMode} style={{ marginLeft: '0.5rem' }}>
          Toggle Mode
        </Button>
      </div>
      <div style={{ marginTop: '0.5rem' }}>
        {mode === 'push' ? (
          <Button
            onMouseDown={startListening}
            onMouseUp={stopListening}
            onTouchStart={startListening}
            onTouchEnd={stopListening}
          >
            Hold to Talk
          </Button>
        ) : (
          <Button onClick={listening ? stopListening : startListening}>
            {listening ? 'Stop Listening' : 'Start Listening'}
          </Button>
        )}
      </div>
      <Waveform active={listening} />
      {transcript && (
        <div style={{ marginTop: '0.5rem' }}>
          "{transcript}" ({Math.round(confidence * 100)}%)
        </div>
      )}
      <HistoryList>
        {history.map((h, i) => (
          <li key={i}>
            {h.text} ({Math.round(h.confidence * 100)}%)
          </li>
        ))}
      </HistoryList>
    </Wrapper>
  );
};

export default VoiceControls;

