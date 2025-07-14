import React, { useState } from 'react';
import styled from '@emotion/styled';
import Panel from './common/Panel';
import Button from './common/Button';

const Wrapper = styled(Panel)`
  display: flex;
  flex-direction: column;
  align-items: center;
`;

const TARSBody = styled.div`
  width: 60px;
  height: 150px;
  background: rgba(255, 255, 255, 0.1);
  position: relative;
  transform-style: preserve-3d;
  animation: rotate 8s linear infinite;

  @keyframes rotate {
    from {
      transform: rotateY(0deg);
    }
    to {
      transform: rotateY(360deg);
    }
  }
`;

const SliderRow = styled.div`
  display: flex;
  width: 100%;
  justify-content: space-between;
  margin-top: 0.5rem;
`;

const Indicator = styled.div<{ active: boolean }>`
  width: 12px;
  height: 12px;
  border-radius: 50%;
  background: ${({ active }) => (active ? 'var(--color-primary)' : '#444')};
  margin-left: 0.5rem;
`;

const Status = styled.div`
  display: flex;
  flex-direction: column;
  width: 100%;
  margin-top: 0.5rem;
  font-size: 0.9rem;
`;

const StatusItem = styled.div`
  display: flex;
  justify-content: space-between;
`;

const TARSInterface: React.FC = () => {
  const [humor, setHumor] = useState(50);
  const [honesty, setHonesty] = useState(90);
  const [voiceActive, setVoiceActive] = useState(false);

  return (
    <Wrapper>
      <TARSBody />
      <SliderRow>
        <label>
          Humor
          <input
            type="range"
            min={0}
            max={100}
            value={humor}
            onChange={(e) => setHumor(Number(e.target.value))}
          />
        </label>
        <label>
          Honesty
          <input
            type="range"
            min={0}
            max={100}
            value={honesty}
            onChange={(e) => setHonesty(Number(e.target.value))}
          />
        </label>
      </SliderRow>
      <div style={{ display: 'flex', alignItems: 'center', marginTop: '0.5rem' }}>
        Voice Activity
        <Indicator active={voiceActive} />
        <Button onClick={() => setVoiceActive((v) => !v)} style={{ marginLeft: '1rem' }}>
          Toggle
        </Button>
      </div>
      <Status>
        <StatusItem>
          <span>Battery:</span> <span>100%</span>
        </StatusItem>
        <StatusItem>
          <span>Connection:</span> <span>Connected</span>
        </StatusItem>
        <StatusItem>
          <span>Mode:</span> <span>Normal</span>
        </StatusItem>
      </Status>
    </Wrapper>
  );
};

export default TARSInterface;

