import React, { useEffect, useRef, useState } from 'react';
import * as THREE from 'three';
import { GLTFLoader } from 'three/examples/jsm/loaders/GLTFLoader.js';
import { VRMLoaderPlugin, VRM, VRMExpressionPresetName } from '@pixiv/three-vrm';
import type { EmotionData, RelationshipData } from '../../types/character';
import { Sparkles, Eye, Heart } from 'lucide-react';

interface VrmAvatarStageProps {
  emotion: EmotionData;
  relationship: RelationshipData;
  interactionStatus: 'idle' | 'thinking' | 'speaking' | 'listening';
  characterName: string;
  isAudioPlaying?: boolean;
  audioAmplitude?: number;
  onSwitchToOrb?: () => void;
}

export const VrmAvatarStage: React.FC<VrmAvatarStageProps> = ({
  emotion,
  relationship,
  interactionStatus,
  characterName,
  isAudioPlaying = false,
  audioAmplitude = 0,
  onSwitchToOrb,
}) => {
  const containerRef = useRef<HTMLDivElement>(null);
  const canvasRef = useRef<HTMLCanvasElement>(null);
  const vrmRef = useRef<VRM | null>(null);
  const sceneRef = useRef<THREE.Scene | null>(null);
  const cameraRef = useRef<THREE.PerspectiveCamera | null>(null);
  const rendererRef = useRef<THREE.WebGLRenderer | null>(null);
  const lookAtTargetRef = useRef<THREE.Object3D | null>(null);
  const animationFrameRef = useRef<number | null>(null);

  const [isLoading, setIsLoading] = useState(true);
  const [loadProgress, setLoadProgress] = useState(0);
  const [loadError, setLoadError] = useState<string | null>(null);

  // Dynamic emotional accent color
  const getEmotionColor = (name: string) => {
    switch (name.toLowerCase()) {
      case 'joy':
      case 'excited':
        return '#ff9f43';
      case 'affection':
        return '#ff6b81';
      case 'sadness':
      case 'melancholic':
        return '#a29bfe';
      case 'anger':
      case 'agitated':
        return '#ff5252';
      case 'surprise':
        return '#feca57';
      default:
        return '#00d2d3';
    }
  };

  const accentColor = getEmotionColor(emotion.dominant_emotion || 'curiosity');

  useEffect(() => {
    if (!containerRef.current || !canvasRef.current) return;

    const container = containerRef.current;
    const canvas = canvasRef.current;
    let isDisposed = false;

    // 1. Three.js Scene Setup
    const scene = new THREE.Scene();
    sceneRef.current = scene;

    const width = container.clientWidth || 320;
    const height = container.clientHeight || 420;

    const camera = new THREE.PerspectiveCamera(30, width / height, 0.1, 20.0);
    // Position camera to frame head and upper torso
    camera.position.set(0.0, 1.35, 1.35);
    camera.lookAt(new THREE.Vector3(0.0, 1.25, 0.0));
    cameraRef.current = camera;

    const renderer = new THREE.WebGLRenderer({
      canvas,
      alpha: true,
      antialias: true,
      powerPreference: 'high-performance',
    });
    renderer.setSize(width, height);
    renderer.setPixelRatio(Math.min(window.devicePixelRatio, 2));
    renderer.outputColorSpace = THREE.SRGBColorSpace;
    rendererRef.current = renderer;

    // 2. Lighting setup for anime MToon shaders
    const ambientLight = new THREE.AmbientLight(0xffffff, 1.4);
    scene.add(ambientLight);

    const dirLight = new THREE.DirectionalLight(0xffffff, 1.8);
    dirLight.position.set(1.0, 2.0, 1.5).normalize();
    scene.add(dirLight);

    const fillLight = new THREE.DirectionalLight(0xb0d4ff, 0.8);
    fillLight.position.set(-1.0, 1.0, 1.0).normalize();
    scene.add(fillLight);

    // LookAt Target
    const lookAtTarget = new THREE.Object3D();
    lookAtTarget.position.set(0.0, 1.3, 2.0);
    scene.add(lookAtTarget);
    lookAtTargetRef.current = lookAtTarget;

    // 3. Load VRM model with progress tracking
    const loader = new GLTFLoader();
    loader.register((parser) => new VRMLoaderPlugin(parser));

    const modelUrl = '/models/avatar.vrm';
    loader.load(
      modelUrl,
      (gltf) => {
        if (isDisposed) return;
        const vrm = gltf.userData.vrm as VRM;
        if (!vrm) {
          setLoadError('Tập tin không phải là định dạng VRM hợp lệ.');
          setIsLoading(false);
          return;
        }

        vrmRef.current = vrm;
        scene.add(vrm.scene);

        // Standard VRM facing orientation
        vrm.scene.rotation.y = Math.PI;

        // Configure lookAt target
        if (vrm.lookAt) {
          vrm.lookAt.target = lookAtTarget;
        }

        setIsLoading(false);
      },
      (progress) => {
        if (progress.total > 0) {
          const pct = Math.round((progress.loaded / progress.total) * 100);
          setLoadProgress(pct);
        }
      },
      (error) => {
        console.error('Lỗi nạp mô hình VRM Avatar:', error);
        setLoadError('Không thể tải mô hình avatar 3D.');
        setIsLoading(false);
      }
    );

    // 4. Handle Resize
    const handleResize = () => {
      if (!containerRef.current || !rendererRef.current || !cameraRef.current) return;
      const w = containerRef.current.clientWidth || 320;
      const h = containerRef.current.clientHeight || 420;
      cameraRef.current.aspect = w / h;
      cameraRef.current.updateProjectionMatrix();
      rendererRef.current.setSize(w, h);
    };

    window.addEventListener('resize', handleResize);

    // 5. Mouse tracking for Look-At
    const handleMouseMove = (e: MouseEvent) => {
      if (!lookAtTargetRef.current) return;
      const ndcX = (e.clientX / window.innerWidth) * 2 - 1;
      const ndcY = -(e.clientY / window.innerHeight) * 2 + 1;

      // Smoothly steer gaze towards cursor
      lookAtTargetRef.current.position.x = ndcX * 0.8;
      lookAtTargetRef.current.position.y = 1.3 + ndcY * 0.4;
      lookAtTargetRef.current.position.z = 1.8;
    };

    window.addEventListener('mousemove', handleMouseMove);

    // 6. Animation Loop (Breathing, Blinking, Lip-Sync, Emotion Interpolation)
    const clock = new THREE.Clock();
    let nextBlinkTime = 3.0;
    let blinkProgress = 0;
    let isBlinking = false;
    let mouthOpenLerp = 0;

    // Emotion weights current state
    const currentExpressions: Record<string, number> = {
      happy: 0,
      sad: 0,
      angry: 0,
      surprised: 0,
      relaxed: 0,
    };

    const animate = () => {
      animationFrameRef.current = requestAnimationFrame(animate);
      const delta = clock.getDelta();
      const elapsed = clock.getElapsedTime();

      const vrm = vrmRef.current;
      if (vrm) {
        vrm.update(delta);

        // A. Idle Breathing: Subtle oscillation of spine/chest
        const chest = vrm.humanoid?.getNormalizedBoneNode('chest') || vrm.humanoid?.getNormalizedBoneNode('spine');
        if (chest) {
          const breath = Math.sin(elapsed * 2.2) * 0.018;
          chest.rotation.x = breath;
        }

        // B. Natural Blinking State Machine
        if (!isBlinking && elapsed > nextBlinkTime) {
          isBlinking = true;
          blinkProgress = 0;
        }

        if (isBlinking) {
          blinkProgress += delta * 12.0; // blink duration ~160ms
          let blinkWeight = 0;
          if (blinkProgress <= 1.0) {
            blinkWeight = blinkProgress;
          } else if (blinkProgress <= 2.0) {
            blinkWeight = 2.0 - blinkProgress;
          } else {
            isBlinking = false;
            blinkWeight = 0;
            nextBlinkTime = elapsed + 3.0 + Math.random() * 2.5; // Next blink in 3-5.5s
          }
          vrm.expressionManager?.setValue(VRMExpressionPresetName.Blink, Math.max(0, Math.min(1, blinkWeight)));
        }

        // C. Real-Time Lip-Sync: Mapped to TTS audio amplitude
        let targetMouth = 0;
        if (isAudioPlaying || interactionStatus === 'speaking') {
          // Responsive opening based on sound energy + slight jitter
          targetMouth = Math.min(1.0, Math.max(0.15, audioAmplitude * 2.2));
        }
        mouthOpenLerp += (targetMouth - mouthOpenLerp) * Math.min(1.0, delta * 18.0);
        vrm.expressionManager?.setValue(VRMExpressionPresetName.Aa, mouthOpenLerp);

        // D. Emotion Blendshape Mapping with Smooth Interpolation
        const dom = (emotion.dominant_emotion || 'curiosity').toLowerCase();
        const intensity = Math.min(1.0, Math.max(0.1, emotion.dominant_intensity ?? 0.6));

        const targetExpressions: Record<string, number> = {
          happy: 0,
          sad: 0,
          angry: 0,
          surprised: 0,
          relaxed: 0,
        };

        if (dom === 'joy' || dom === 'excited') {
          targetExpressions.happy = intensity * 0.9;
        } else if (dom === 'affection') {
          targetExpressions.happy = intensity * 0.6;
          targetExpressions.relaxed = 0.4;
        } else if (dom === 'sadness' || dom === 'melancholic') {
          targetExpressions.sad = intensity * 0.85;
        } else if (dom === 'anger' || dom === 'agitated') {
          targetExpressions.angry = intensity * 0.9;
        } else if (dom === 'surprise') {
          targetExpressions.surprised = intensity * 0.85;
        } else {
          targetExpressions.relaxed = 0.3;
        }

        const lerpSpeed = Math.min(1.0, delta * 4.0);
        for (const key of Object.keys(currentExpressions)) {
          currentExpressions[key] += (targetExpressions[key] - currentExpressions[key]) * lerpSpeed;
        }

        if (vrm.expressionManager) {
          vrm.expressionManager.setValue(VRMExpressionPresetName.Happy, currentExpressions.happy);
          vrm.expressionManager.setValue(VRMExpressionPresetName.Sad, currentExpressions.sad);
          vrm.expressionManager.setValue(VRMExpressionPresetName.Angry, currentExpressions.angry);
          vrm.expressionManager.setValue(VRMExpressionPresetName.Surprised, currentExpressions.surprised);
          vrm.expressionManager.setValue(VRMExpressionPresetName.Relaxed, currentExpressions.relaxed);
        }
      }

      renderer.render(scene, camera);
    };

    animate();

    return () => {
      isDisposed = true;
      window.removeEventListener('resize', handleResize);
      window.removeEventListener('mousemove', handleMouseMove);
      if (animationFrameRef.current) {
        cancelAnimationFrame(animationFrameRef.current);
      }
      renderer.dispose();
    };
  }, [interactionStatus, isAudioPlaying, audioAmplitude, emotion]);

  return (
    <div
      ref={containerRef}
      style={{
        position: 'relative',
        width: '100%',
        height: '420px',
        display: 'flex',
        flexDirection: 'column',
        alignItems: 'center',
        justifyContent: 'center',
        overflow: 'hidden',
        userSelect: 'none',
      }}
    >
      {/* Three.js Transparent Canvas */}
      <canvas
        ref={canvasRef}
        style={{
          width: '100%',
          height: '100%',
          display: 'block',
          outline: 'none',
        }}
      />

      {/* Loading Overlay */}
      {isLoading && !loadError && (
        <div
          style={{
            position: 'absolute',
            inset: 0,
            display: 'flex',
            flexDirection: 'column',
            alignItems: 'center',
            justifyContent: 'center',
            background: 'rgba(8, 10, 16, 0.75)',
            backdropFilter: 'blur(8px)',
            zIndex: 10,
          }}
        >
          <div
            style={{
              width: '44px',
              height: '44px',
              borderRadius: '50%',
              border: `3px solid rgba(255, 255, 255, 0.1)`,
              borderTopColor: accentColor,
              animation: 'spin 1s linear infinite',
              marginBottom: '14px',
            }}
          />
          <div style={{ color: '#fff', fontSize: '13px', fontWeight: 600, letterSpacing: '0.05em' }}>
            Đang khởi tạo 3D VRM Avatar... {loadProgress > 0 ? `${loadProgress}%` : ''}
          </div>
          <div style={{ color: 'rgba(255, 255, 255, 0.45)', fontSize: '11px', marginTop: '6px' }}>
            Chuẩn bị biểu cảm khuôn mặt & Lip-Sync
          </div>
        </div>
      )}

      {/* Error Fallback Notice */}
      {loadError && (
        <div
          style={{
            position: 'absolute',
            inset: 0,
            display: 'flex',
            flexDirection: 'column',
            alignItems: 'center',
            justifyContent: 'center',
            background: 'rgba(8, 10, 16, 0.85)',
            padding: '20px',
            textAlign: 'center',
            zIndex: 10,
          }}
        >
          <p style={{ color: '#ff6b6b', fontSize: '13px', marginBottom: '10px' }}>{loadError}</p>
          {onSwitchToOrb && (
            <button
              onClick={onSwitchToOrb}
              style={{
                background: 'rgba(255, 255, 255, 0.1)',
                border: '1px solid rgba(255, 255, 255, 0.2)',
                color: '#fff',
                padding: '8px 16px',
                borderRadius: '8px',
                fontSize: '12px',
                cursor: 'pointer',
              }}
            >
              Chuyển về Hào quang Cảm xúc
            </button>
          )}
        </div>
      )}

      {/* Stage Floating HUD Badges */}
      <div
        style={{
          position: 'absolute',
          top: '12px',
          left: '16px',
          right: '16px',
          display: 'flex',
          justifyContent: 'space-between',
          alignItems: 'center',
          pointerEvents: 'none',
        }}
      >
        {/* Left: Avatar Mode Indicator */}
        <div
          style={{
            display: 'inline-flex',
            alignItems: 'center',
            gap: '6px',
            padding: '4px 10px',
            borderRadius: '20px',
            background: 'rgba(15, 20, 32, 0.65)',
            border: '1px solid rgba(255, 255, 255, 0.12)',
            backdropFilter: 'blur(10px)',
            fontSize: '11px',
            color: '#fff',
            fontWeight: 500,
          }}
        >
          <Sparkles size={12} color={accentColor} />
          <span>3D VRM Live Stage</span>
        </div>

        {/* Right: View Toggle Button (switchable to Orb mode) */}
        {onSwitchToOrb && (
          <button
            onClick={onSwitchToOrb}
            style={{
              pointerEvents: 'auto',
              display: 'inline-flex',
              alignItems: 'center',
              gap: '6px',
              padding: '4px 10px',
              borderRadius: '20px',
              background: 'rgba(255, 255, 255, 0.08)',
              border: '1px solid rgba(255, 255, 255, 0.15)',
              backdropFilter: 'blur(10px)',
              fontSize: '11px',
              color: 'rgba(255, 255, 255, 0.8)',
              cursor: 'pointer',
              transition: 'all 0.2s ease',
            }}
            title="Chuyển sang chế độ Hào quang cảm xúc dạng Cầu năng lượng"
          >
            <Eye size={12} />
            <span>Chế độ Cầu Năng Lượng</span>
          </button>
        )}
      </div>

      {/* Bottom Status Ribbon */}
      <div
        style={{
          position: 'absolute',
          bottom: '12px',
          display: 'flex',
          alignItems: 'center',
          gap: '8px',
          padding: '6px 14px',
          borderRadius: '24px',
          background: 'rgba(15, 20, 32, 0.7)',
          border: `1px solid ${accentColor}40`,
          backdropFilter: 'blur(12px)',
          boxShadow: `0 4px 20px ${accentColor}25`,
        }}
      >
        <span
          style={{
            width: '8px',
            height: '8px',
            borderRadius: '50%',
            backgroundColor: accentColor,
            boxShadow: `0 0 8px ${accentColor}`,
            animation: interactionStatus === 'speaking' ? 'pulse 0.8s infinite' : 'none',
          }}
        />
        <span style={{ fontSize: '12px', fontWeight: 600, color: '#fff' }}>
          {characterName}
        </span>
        <span
          style={{
            fontSize: '10px',
            color: 'rgba(255, 255, 255, 0.7)',
            display: 'inline-flex',
            alignItems: 'center',
            gap: '3px',
            background: 'rgba(255, 255, 255, 0.1)',
            padding: '2px 6px',
            borderRadius: '10px',
          }}
        >
          <Heart size={9} color="#ff758c" />
          <span>{relationship.stage}</span>
        </span>
        <span style={{ fontSize: '11px', color: 'rgba(255, 255, 255, 0.55)' }}>
          {interactionStatus === 'speaking'
            ? 'Đang nói...'
            : interactionStatus === 'thinking'
            ? 'Đang suy nghĩ...'
            : interactionStatus === 'listening'
            ? 'Đang nghe...'
            : 'Sẵn sàng'}
        </span>
      </div>
    </div>
  );
};
