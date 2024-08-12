#version 130

uniform vec4 albedoColor;

uniform vec3 eyePosition;

uniform float albedoTextureFactor;
uniform sampler2D albedoTexture;
uniform float normalTextureFactor;
uniform sampler2D normalTexture;
uniform float displacementTextureFactor;
uniform sampler2D displacementTexture;
uniform float specularTextureFactor;
uniform sampler2D specularTexture;

uniform mat4 viewMatrix;

in vec3 vNormal;
in vec2 vTexCoords;
in vec4 vWorldPos;
in mat3 vTangentMatrix;
in mat3 vInvTangentMatrix;

vec3 viewDir;
vec3 normal;
vec2 texCoords;

float pointLightDiffuse(vec3 lp)
{
	return 1.0f;
}

float pointLightSpecular(vec3 lp)
{
	return 1.0f;
}

float directionalLightDiffuse(vec3 ld)
{
	return clamp(dot(normal, -normalize(ld)), 0.0f, 1.0f);
}

float directionalLightSpecular(vec3 ld, float shininess)
{
	if (shininess == 0.0f)
	{
		return 0.0f;
	}
	else
	{
		vec3 h = normalize(viewDir) + normalize(ld);
		return (shininess / 128.0f) * max(0.0f, pow(clamp(dot(normal, -normalize(h)), 0.0, 1.0), shininess));
//		return pow(max(0.0f, clamp(dot(normal, -normalize(h)), 0.0, 1.0)), shininess);
	}
}

// tsViewDir: texture space view dir
vec2 parallaxMapping(in vec3 tsViewDir, in vec2 texCoords, in float layers, in float parallaxScale, out float parallaxHeight)
{
	// determine optimal number of layers
	const float minLayers = 10.0f;
	const float maxLayers = 15.0f;
	float numLayers = mix(maxLayers, minLayers, abs(dot(vec3(0.0f, 0.0f, 1.0f), tsViewDir)));
	numLayers = layers;

	// height of each layer
	float layerHeight = 1.0f / numLayers;
	// current depth of the layer
	float curLayerHeight = 0.0f;
	// shift of texture coordinates for each layer
	vec2 dtex = parallaxScale * tsViewDir.xy / tsViewDir.z / numLayers;

	// current texture coordinates
	vec2 currentTextureCoords = texCoords;

	// depth from heightmap
	float heightFromTexture = texture(displacementTexture, currentTextureCoords).r;

	// while point is above the surface
	while(heightFromTexture > curLayerHeight)
	{
		// to the next layer
		curLayerHeight += layerHeight;
		// shift of texture coordinates
		currentTextureCoords += dtex;
		// new depth from heightmap
		heightFromTexture = texture(displacementTexture, currentTextureCoords).r;
	}

	///////////////////////////////////////////////////////////

	// previous texture coordinates
	vec2 prevTCoords = currentTextureCoords - dtex;

	// heights for linear interpolation
	float nextH = heightFromTexture - curLayerHeight;
	float prevH = texture(displacementTexture, prevTCoords).r - curLayerHeight + layerHeight;

	// proportions for linear interpolation
	float weight = nextH / (nextH - prevH);

	// interpolation of texture coordinates
	vec2 finalTexCoords = prevTCoords * weight + currentTextureCoords * (1.0f - weight);

	// interpolation of depth values
	parallaxHeight = curLayerHeight + prevH * weight + nextH * (1.0f - weight);

	// return result
	return finalTexCoords;
}

void main()
{
	viewDir = normalize(vWorldPos.xyz - eyePosition);

	float parallaxHeight;
	if (displacementTextureFactor == 1.0f)
	{
		vec3 tsViewDir = vInvTangentMatrix * viewDir;
		texCoords = parallaxMapping(tsViewDir, vTexCoords, 50, 0.05, parallaxHeight);
	}
	else
	{
		texCoords = vTexCoords;
	}

	normal = normalize(vNormal);
	normal = vec3(0.0f, 0.0f, 1.0f);
	normal = normal * (1.0f - normalTextureFactor) + (texture(normalTexture, texCoords).rgb * 2.0f - 1.0f) * normalTextureFactor;
	normal = normalize(vTangentMatrix * normal);

	vec4 ambient = vec4(0.1f, 0.1f, 0.1f, 0.0f);

	vec4 albedo = albedoColor;
	albedo = albedo * (1.0f - albedoTextureFactor) + texture(albedoTexture, texCoords) * albedoTextureFactor;

	float alpha = albedo.a;

	float shininess = texture(specularTexture, texCoords).r * specularTextureFactor * 128.0f;

	vec4 lightDiffuseIntensity = vec4(0.1f, 0.1f, 0.1f, 0.1f);
	vec4 lightSpecularIntensity = vec4(0.0f, 0.0f, 0.0f, 0.0f);

	lightDiffuseIntensity += vec4(1.0f, 1.0f, 1.0f, 1.0f) * directionalLightDiffuse(vec3(1.0f, 1.0f, -1.0f));
	lightSpecularIntensity += vec4(1.0f, 1.0f, 1.0f, 1.0f) * directionalLightSpecular(vec3(1.0f, 1.0f, -1.0f), shininess);

//	lightDiffuseIntensity += vec4(0.1f, 0.1f, 0.4f, 1.0f) * directionalLightDiffuse(vec3(-1.0f, -1.0f, -1.0f));
//	lightSpecularIntensity += vec4(0.1f, 0.1f, 0.4f, 1.0f) * directionalLightSpecular(vec3(-1.0f, -1.0f, -1.0f), shininess);

	vec4 resultColor;

	// albedo
//	resultColor = albedo;

	// ambient
//	resultColor = ambient * albedoColor;

	// ambient + diffuse
//	resultColor = albedo * (ambient + lightDiffuseIntensity);

	// ambient + diffuse + specular
	resultColor = max(albedo * (ambient + lightDiffuseIntensity), lightSpecularIntensity);

	// ambient + diffuse + specular + normal map

	// ambient + diffuse + specular + normal map + displacement map

	resultColor.a = alpha;
	gl_FragColor = resultColor;
}
