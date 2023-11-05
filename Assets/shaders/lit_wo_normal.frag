#version 400

const int maxUvChannelCount = 10;
const int maxBoneCount = 50;

uniform vec3 eyePosition;
uniform mat4 objectMatrix;
uniform mat4 viewMatrix;
uniform mat4 projectionMatrix;
uniform mat4 normalMatrix;
uniform mat4 bones[maxBoneCount];

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

in vec4 vWorldPos;
in vec3 vNormal;
in vec2 vUvChannels[maxUvChannelCount];
in mat3 vTangentMatrix;
in mat3 vInvTangentMatrix;

out vec4 fragColor;

vec4 getAlbedoColor(vec2 texCoordsOffset) {
	if (useAlbedoTexture == 1) {
		return texture(
			albedoTexture,
			vUvChannels[albedoTextureUvChannelId] + texCoordsOffset
		);
	} else {
		return vec4(1.0f);
	}
}

vec3 getNormal(vec2 texCoordsOffset) {
	return vNormal;
}

void main() {
	vec3 viewDir = normalize(vWorldPos.xyz - eyePosition);

	vec2 texCoordsOffset = vec2(0.0f);

	vec3 albedo = vec3(1.0f, 1.0f, 1.0f);
	vec3 ambient = vec3(0.1f, 0.1f, 0.1f);

	vec4 tmp = getAlbedoColor(texCoordsOffset);
	albedo = vec3(tmp);
	float alpha = tmp.a;

	vec3 normal = getNormal(texCoordsOffset);

	vec3 lightIntensity = vec3(0.1f, 0.1f, 0.1f);

	vec3 lightDir0 = vec3(1.2f, -0.8f, -1.0f);
	vec3 lightColor0 = vec3(1.0f, 1.0f, 1.0f);
	lightIntensity += lightColor0 * clamp(dot(normal, -normalize(lightDir0)), 0.0f, 1.0f);

	vec3 lightDir1 = vec3(-1.0f, 1.0f, 1.0f);
	vec3 lightColor1 = vec3(0.1f, 0.1f, 0.4f);
	lightIntensity += lightColor1 * clamp(dot(normal, -normalize(lightDir1)), 0.0f, 1.0f);

	vec3 resultColor = (albedo * lightIntensity + ambient) * albedoColor;
	fragColor = max(vec4(resultColor, alpha), vec4(emissiveColor, alpha));
}
