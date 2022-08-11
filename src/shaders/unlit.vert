#version 400

const int maxUvChannelCount = 10;
const int maxBoneCount = 50;

in vec3 position;
in vec3 normal;
in vec3 tangent;
in vec2 uvChannels[maxUvChannelCount];
in vec4 boneIds;
in vec4 boneWeights;

uniform vec3 eyePosition;
uniform mat4 objectMatrix;
uniform mat4 viewMatrix;
uniform mat4 projectionMatrix;
uniform mat3 normalMatrix;
uniform mat4 bones;

uniform int useAlbedoTexture;
uniform sampler2D albedoTexture;
uniform uint albedoTextureUvChannelId;

uniform int useNormalTexture;
uniform sampler2D normalTexture;
uniform uint normalTextureUvChannelId;

uniform int useDisplacementTexture;
uniform sampler2D displacementTexture;
uniform uint displacementTextureUvChannelId;

uniform float opacity;
uniform vec3 albedoColor;
uniform vec3 emissiveColor;
uniform vec3 shininessColor;

void main()
{
	gl_Position = objectMatrix * vec4(position, 1.0f);
}
