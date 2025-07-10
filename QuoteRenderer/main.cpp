#include <iostream>
#include <cmath>

#include "renderer.h"
#include "helpers/rounded_rect.h"

#define SCALE 2


int main(int argc, char* argv[])
{
    quote::renderer renderer;
    const auto rect = new sf::RoundedRectangleShape {sf::Vector2f {400, 200}, 20, 20};
    const auto circle = new sf::CircleShape {30 * SCALE, 100};
    
    sf::Font open_sans;
    sf::Font noto;
    open_sans.loadFromFile("OpenSans-VariableFont.ttf");
    const auto username = new sf::Text{L"Olga Sulema", open_sans, 24 * SCALE};
    const auto message = new sf::Text{L"Hello world!", open_sans, 24 * SCALE};
    username->setStyle(sf::Text::Bold);
    username->setFillColor(sf::Color::Black);
    message->setFillColor(sf::Color::Black);
    const auto text_size = std::max(message->getLocalBounds().width, username->getLocalBounds().width);
    sf::Texture tex;
    tex.setSmooth(true);
    tex.loadFromFile("hamster.jpg");
    rect->setSize(sf::Vector2f(std::max(text_size + 20.f, 160.f * SCALE), (58 + 24) * SCALE));
    rect->setPosition(70 * SCALE, 0);
    circle->setTexture(&tex);
    username->setPosition(80 * SCALE, 10 * SCALE);
    message->setPosition(80 * SCALE, 13 * SCALE + username->getLocalBounds().height);
    
    renderer.add_drawable(circle);    
    renderer.add_drawable(rect);
    renderer.add_drawable(username);
    renderer.add_drawable(message);
    renderer.render(800, 600);
    // char c {};
    // std::cin >> c;
    return 0;
}
