nightshade_config::config! {
    pub enum Settings {
        "Ground" {
            GroundBase: number(-200.0, 0.0, -60.0),
            GroundSize: number(50.0, 800.0, 300.0),
            GroundOctaves: whole(1, 12, 10),
            GroundGain: number(0.1, 0.7, 0.41),
            GroundClip: number(0.0, 0.5, 0.1),
            GroundClipSmooth: number(0.0, 0.2, 0.05),
            GroundJoinSmooth: number(0.0, 0.2, 0.055),
            GroundSkin: number(0.0, 0.5, 0.05),
            GroundTint: color([0.43, 0.43, 0.43, 1.0]),
            GroundDirt: color([0.18, 0.07, 0.01, 1.0]),
            GroundRoughness: number(0.04, 1.0, 0.85),
            GroundFriction: number(0.0, 2.0, 1.0),
        }
        "Camera" {
            CameraSpeed: number(2.0, 60.0, 14.0),
            CameraClearance: number(1.0, 20.0, 6.0),
            MarchScale: number(0.25, 1.0, 0.5),
            CollisionReach: number(4.0, 40.0, 13.0),
        }
        "Blobs" {
            BlobCount: whole(0, 16, 7),
            BlobOrbit: number(1.0, 20.0, 6.0),
            BlobHeight: number(0.0, 20.0, 8.5),
            BlobBob: number(0.0, 5.0, 1.8),
            BlobRadius: number(0.3, 4.0, 1.7),
            BlobBlend: number(0.0, 3.0, 1.4),
            BlobDrift: number(-2.0, 2.0, 0.35),
            BlobTint: color([0.86, 0.42, 0.30, 1.0]),
            BlobRoughness: number(0.04, 1.0, 0.35),
        }
        "Cubes" {
            CubeCount: whole(0, 16, 5),
            CubeOrbit: number(1.0, 30.0, 13.0),
            CubeHeight: number(0.0, 20.0, 4.0),
            CubeBob: number(0.0, 5.0, 0.9),
            CubeHalf: number(0.2, 4.0, 1.3),
            CubeRounding: number(0.0, 1.0, 0.18),
            CubeBlend: number(0.0, 3.0, 0.7),
            CubeDrift: number(-2.0, 2.0, -0.18),
            CubeSpin: number(-3.0, 3.0, 0.7),
            CubeTint: color([0.30, 0.55, 0.82, 1.0]),
            CubeRoughness: number(0.04, 1.0, 0.25),
            CubeMetal: number(0.0, 1.0, 0.0),
        }
        "Ring" {
            RingMajor: number(1.0, 10.0, 4.5),
            RingMinor: number(0.2, 3.0, 1.1),
            RingHeight: number(0.0, 30.0, 13.0),
            RingSpin: number(-3.0, 3.0, 0.45),
            RingTint: color([0.95, 0.72, 0.28, 1.0]),
            RingRoughness: number(0.04, 1.0, 0.22),
            RingMetal: number(0.0, 1.0, 1.0),
        }
        "Props" {
            BarCount: whole(0, 24, 5),
            BarLength: number(0.5, 5.0, 1.8),
            BarWidth: number(0.2, 3.0, 0.8),
            BarTint: color([0.95, 0.66, 0.16, 1.0]),
            BarRoughness: number(0.04, 1.0, 0.3),
            BlockCount: whole(0, 24, 7),
            BlockSize: number(0.2, 3.0, 0.55),
            BlockTint: color([0.88, 0.91, 0.95, 1.0]),
            BlockRoughness: number(0.04, 1.0, 0.18),
            SpikeCount: whole(0, 24, 3),
            SpikeRadius: number(0.3, 4.0, 1.5),
            SpikeTint: color([1.0, 0.36, 0.06, 1.0]),
            SpikeRoughness: number(0.04, 1.0, 0.28),
            PropRounding: number(0.0, 0.5, 0.06),
            PropCenter: number(0.0, 60.0, 22.0),
            PropSpread: number(2.0, 30.0, 11.0),
            PropDrop: number(0.5, 20.0, 3.0),
            PropFocus: number(1.0, 12.0, 4.0),
        }
        "Hole" {
            HoleRadius: number(0.5, 8.0, 2.4),
            HoleDepth: number(1.0, 20.0, 6.0),
            HoleBlend: number(0.0, 2.0, 0.4),
        }
        "Tunnel" {
            TunnelHalfHeight: number(2.0, 60.0, 22.0),
            TunnelRadius: number(0.3, 6.0, 1.6),
            TunnelBlend: number(0.0, 2.0, 0.35),
            TunnelReach: number(2.0, 60.0, 22.0),
        }
        "Carve" {
            CarveRadius: number(0.2, 4.0, 0.8),
            CarveBlend: number(0.0, 2.0, 0.2),
            CarveSpeed: number(0.2, 10.0, 1.6),
        }
        "Build" {
            BuildRadius: number(0.2, 4.0, 0.6),
            BuildBlend: number(0.0, 2.0, 0.25),
            BuildSpeed: number(0.2, 10.0, 1.2),
            BuildClearance: number(0.5, 10.0, 2.0),
            BuildTint: color([0.82, 0.62, 0.40, 1.0]),
            BuildRoughness: number(0.04, 1.0, 0.6),
        }
        "Bite" {
            BiteSeed: number(0.05, 1.0, 0.2),
            BiteDrift: number(0.1, 1.0, 0.5),
        }
        "Stroke" {
            StrokeRadius: number(0.05, 1.0, 0.2),
            StrokeBlend: number(0.0, 1.0, 0.12),
            StrokeSpacing: number(0.03, 1.0, 0.15),
            StrokeJump: number(0.2, 10.0, 1.6),
            StrokeTint: color([0.28, 0.58, 1.0, 1.0]),
            StrokeRoughness: number(0.04, 1.0, 0.12),
        }
        "Stones" {
            StoneCount: whole(0, 24, 5),
            StoneRadius: number(0.2, 3.0, 0.85),
            StoneBlend: number(0.0, 2.0, 0.45),
            StoneTint: color([0.55, 0.62, 0.48, 1.0]),
            StoneRoughness: number(0.04, 1.0, 0.5),
            StoneAhead: number(1.0, 20.0, 4.0),
            StoneSpeed: number(1.0, 60.0, 16.0),
            StoneSpread: number(0.5, 10.0, 3.0),
            StoneFriction: number(0.0, 2.0, 0.9),
            StoneRestitution: number(0.0, 1.0, 0.25),
            StoneFocus: number(1.0, 12.0, 4.0),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Prop {
    Bar,
    Block,
    Spike,
    Stone,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Tool {
    #[default]
    Carve,
    Build,
    Draw,
    Hole,
}

impl Tool {
    pub const ALL: [Tool; 4] = [Tool::Carve, Tool::Build, Tool::Draw, Tool::Hole];

    pub fn name(self) -> &'static str {
        match self {
            Tool::Carve => "Carve",
            Tool::Build => "Build",
            Tool::Draw => "Draw",
            Tool::Hole => "Hole",
        }
    }
}
