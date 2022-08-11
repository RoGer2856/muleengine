#version 400

out vec4 fragColor;

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
	fragColor = vec4(1.0, 0.6, 0.3, 1.0);
}
